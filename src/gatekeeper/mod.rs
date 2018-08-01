extern crate failure;
mod update_handler;
mod gate_open_handler;

use diesel::{PgConnection};
use database::establish_connection;
use std;
use std::env;
use teleborg::*;
use super::hardware::*;
use crossbeam_channel as channel;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
extern crate sysfs_gpio;
use self::sysfs_gpio::Edge;

pub struct GringosGateKeeperBot<T> where T: TelegramInterface{
    telegram_api: T,
    database_connection: PgConnection,
    hardware: Hardware,
    last_pic_date: std::time::Instant,
    last_pic_path: String,
    instant_when_was_opened: Option<std::time::Instant>,
    sent_open_warning: bool
}

enum NewState {
    Opened,
    Closed
}

enum Event{
    GateStateChange(NewState),
    TelegramMessage(Vec<Update>),
    VerifyOpenTooLong
}

impl <T: TelegramInterface> GringosGateKeeperBot<T>{

    pub fn new() -> Self{
        let bot_token = env::var("TELEGRAM_GATE_BOT_ID")
            .ok()
            .expect("Can't find TELEGRAM_GATE_BOT_ID env variable")
            .parse::<String>()
            .unwrap();
        GringosGateKeeperBot{
            telegram_api: T::new(bot_token).unwrap(),
            database_connection: establish_connection(),
            hardware: Hardware::new(),
            last_pic_date: std::time::Instant::now(),
            last_pic_path: "rep_now.jpg".to_string(),
            instant_when_was_opened: None,
            sent_open_warning: false
        }
    }

    pub fn emergency_turn_off(){
        let hw = Hardware::new();
        hw.turn_off_spotlight();
        hw.close_gate();
    }

    pub fn start(&mut self){
        let (sender, receiver): (Sender<Event>, Receiver<Event>) = channel::unbounded();
        self.telegram_api.start_getting_updates();

        let telegram_update_receiver = self.telegram_api.get_updates_channel().clone();
        let telegram_thread_sender = sender.clone();
        let gate_state_thread_sender = sender.clone();
        std::thread::spawn(move ||{
            while let Some(update) = telegram_update_receiver.recv(){
                telegram_thread_sender.send(Event::TelegramMessage(update));
            }
        });

        self.hardware.gate_open_sensor
            .set_edge(Edge::BothEdges)
            .expect("Sensor pin does not allow interrupts!");

        let mut gate_open_poller = self.hardware.gate_open_sensor
            .get_poller()
            .expect("Could not get pin Poller");


        std::thread::spawn(move ||{
            loop {
                let gpio_value = gate_open_poller.poll(-1)
                    .unwrap_or_else(|e| panic!("Got error {} while polling GPIO pin.", e))
                    .expect("Got none while polling GPIO, this should never happen since it has -1 as timeout");
                if gpio_value == 0 {
                    gate_state_thread_sender.send(Event::GateStateChange(NewState::Opened));
                } else {
                    gate_state_thread_sender.send(Event::GateStateChange(NewState::Closed));
                }
            }
        });

        loop {
            match receiver.recv().expect("Channel was closed! This should never happen!"){
                Event::TelegramMessage(msg_vec) => {
                    for update in msg_vec {
                        self.handle_update(update);
                    }
                },
                Event::GateStateChange(new_state) => {
                    match new_state {
                        NewState::Opened => {
                            self.handle_gate_open();
                        },
                        NewState::Closed => {
                            if self.instant_when_was_opened.is_some(){
                                self.hardware.turn_off_spotlight();
                                self.instant_when_was_opened.take();
                                self.telegram_api.send_msg(OutgoingMessage::new(75698394, "O portão foi fechado!"));
                                self.sent_open_warning = false;
                            }
                        },
                    }
                },
                Event::VerifyOpenTooLong =>{
                    info!("Got to verify!");
                }
            }
        }
    }
}

