extern crate failure;
mod update_handler;
mod gate_open_handler;

extern crate sysfs_gpio;
use diesel::{PgConnection};
use database::establish_connection;
use std;
use std::env;
use teleborg::*;
use super::hardware::*;
use crossbeam_channel as channel;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use self::sysfs_gpio::Edge;
use database::models::CoffeezeraUser;

pub struct GringosGateKeeperBot<T> where T: TelegramInterface{
    telegram_api: T,
    database_connection: PgConnection,
    hardware: Hardware,
    picture_context: PictureContext,
    last_opening_by_bot: Option<LastOpeningData>,
    instant_when_was_opened: Option<std::time::Instant>,
}

pub struct PictureContext{
    last_pic_date: std::time::Instant,
    last_pic_path: String,
}

pub struct LastOpeningData {
    who_last_opened_it: CoffeezeraUser,
    when_user_opened: std::time::Instant,
}


pub enum NewState {
    Opened,
    Closed
}

pub enum Event{
    GateStateChange(NewState),
    TelegramMessage(Vec<Update>),
    VerifyOpenTooLong
}

impl <T: TelegramInterface> GringosGateKeeperBot<T> {
    pub fn new() -> Self {
        let bot_token = env::var("TELEGRAM_GATE_BOT_ID")
            .ok()
            .expect("Can't find TELEGRAM_GATE_BOT_ID env variable")
            .parse::<String>()
            .unwrap();
        GringosGateKeeperBot {
            telegram_api: T::new(bot_token).unwrap(),
            database_connection: establish_connection(),
            hardware: Hardware::new(),
            picture_context: PictureContext{
                last_pic_date: std::time::Instant::now(),
                last_pic_path: "rep_now.jpg".to_string(),
            },
            last_opening_by_bot: None,
            instant_when_was_opened: None,
        }
    }

    pub fn emergency_turn_off() {
        let hw = Hardware::new();
        hw.turn_off_spotlight();
        hw.close_gate();
    }

    pub fn start(&mut self) {
        let (sender, receiver): (Sender<Event>, Receiver<Event>) = channel::unbounded();
        self.telegram_api.start_getting_updates();

        let telegram_update_receiver = self.telegram_api.get_updates_channel().clone();
        let telegram_thread_sender = sender.clone();
        let gate_state_thread_sender = sender.clone();
        std::thread::spawn(move || {
            while let Some(update) = telegram_update_receiver.recv() {
                telegram_thread_sender.send(Event::TelegramMessage(update));
            }
        });



        let gate_open_sensor_clone = self.hardware.gate_open_sensor.clone();


        std::thread::spawn(move || {
            gate_open_sensor_clone
                .set_edge(Edge::BothEdges)
                .expect("Sensor pin does not allow interrupts!");

            let mut gate_open_poller = gate_open_sensor_clone
                .get_poller()
                .expect("Could not get pin Poller");
            let mut last_value_sent = 2;
            'wait_forever_for_gpio_change: loop {
                // Wait until a change occurs
                let mut latest_gpio_value_change = gate_open_poller.poll(-1)
                    .unwrap_or_else(|e| panic!("Got error {} while polling GPIO pin.", e))
                    .expect("Got none while polling GPIO, this should never happen since it has -1 as timeout");
                // loop until value is stable
                'wait_for_gpio_stabilization: loop {
                    info!("GPIO value changed to {}. Waiting for stabilization.", latest_gpio_value_change);
                    // Wait up to 500ms for a second change
                    let maybe_second_gpio_value_change = gate_open_poller.poll(500)
                        .unwrap_or_else(|e| panic!("Got error {} while polling GPIO pin.", e));
                    match maybe_second_gpio_value_change {
                        Some(second_gpio_value_change) => {
                            info!("There was second change in the GPIO value in less than 500ms. Its not stable. New value is {}", second_gpio_value_change);
                            latest_gpio_value_change = second_gpio_value_change;
                            continue 'wait_for_gpio_stabilization;
                        },
                        None => {
                            info!("GPIO did not change for 500ms, declaring it stable");
                            let new_value = gate_open_sensor_clone.get_value()
                                .expect("Could not read value from gate_open_sensor");
                            if new_value != latest_gpio_value_change{
                                error!("Value from stable change is different from actual value: Change = {} Actual = {}", latest_gpio_value_change, new_value);
                            }
                            if new_value == last_value_sent{
                                info!("New stable value is the same as last one. No actual change occured.");
                                continue 'wait_forever_for_gpio_change;
                            }
                            last_value_sent = new_value;
                            if new_value == 0 {
                                info!("Sending new GPIO change: 0");
                                gate_state_thread_sender.send(Event::GateStateChange(NewState::Opened));
                            } else {
                                info!("Sending new GPIO change: 1");
                                gate_state_thread_sender.send(Event::GateStateChange(NewState::Closed));
                            }
                            continue 'wait_forever_for_gpio_change;
                        }
                    }
                }
            }
        });

        loop {
            match receiver.recv().expect("Channel was closed! This should never happen!") {
                Event::TelegramMessage(msg_vec) => {
                    for update in msg_vec {
                        self.handle_update(update);
                    }
                },
                Event::GateStateChange(new_state) => {
                    match new_state {
                        NewState::Opened => {
                            let timer_thread_sender = sender.clone();
                            self.handle_gate_open(timer_thread_sender);
                        },
                        NewState::Closed => {
                            if self.instant_when_was_opened.is_some() {
                                self.hardware.turn_off_spotlight();
                                self.instant_when_was_opened.take();
                                self.telegram_api.send_msg(OutgoingMessage::new(75698394, "O portão foi fechado!"));
                            }
                        },
                    }
                },
                Event::VerifyOpenTooLong => {
                    self.check_gate_open_too_long();
                }
            }
        }
    }
}

