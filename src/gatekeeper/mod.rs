mod telegram_update_handler;
mod gate_open_handler;

use self::gate_open_handler::*;
use diesel::{PgConnection};
use crate::database::establish_connection;
use std;
use std::env;
use teleborg::*;
use super::hardware::*;
use crossbeam_channel as channel;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use sysfs_gpio::Edge;
//use self::gate_open_handler::CHAT_TO_SEND_MSG;
use crate::database::models::CoffeezeraUser;

pub struct GringosGateKeeperBot<T> where T: TelegramInterface{
    telegram_api: T,
    database_connection: PgConnection,
    hardware: Hardware,
    picture_context: PictureContext,
    last_person_opened: Option<LastPersonOpened>,
    last_gate_open_event: Option<LastGateOpenEvent>,
    internal_events_sender: Sender<Event>,
    internal_events_receiver: Option<Receiver<Event>>,
}


pub struct PictureContext{
    last_pic_date: std::time::Instant,
    last_pic_path: String,
}

pub struct LastGateOpenEvent{
    instant_when_was_opened_if_is_open: std::time::Instant,
    turned_on_spot_light: bool
}

pub struct LastPersonOpened {
    who_last_opened_it: CoffeezeraUser,
    when_user_opened: std::time::Instant,
    sent_open_warning: bool,
}



pub enum Event{
    GateStateChange(NewGateState),
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
        let (sender, receiver): (Sender<Event>, Receiver<Event>) = channel::unbounded();
        GringosGateKeeperBot {
            telegram_api: T::new(bot_token).unwrap(),
            database_connection: establish_connection(),
            hardware: Hardware::new(),
            picture_context: PictureContext{
                last_pic_date: std::time::Instant::now(),
                last_pic_path: "rep_now.jpg".to_string(),
            },
            last_person_opened: None,
            last_gate_open_event: None,
            internal_events_sender: sender,
            internal_events_receiver: Some(receiver),
        }
    }

    pub fn emergency_turn_off() {
        let hw = Hardware::new();
        hw.emergency_turn_off_spotlight();
        hw.allow_lock();
    }



    fn start_getting_gate_state_changes(&mut self){
        // get a sender reference clone
        let event_sender = self.internal_events_sender.clone();
        // closure to be executed on event
        let on_gate_state_change = move |new_state: super::hardware::NewGateState|{
            event_sender.send(Event::GateStateChange(new_state)).expect("Could not send gate change event!");
        };
        self.hardware.start_listening_gate_state_change(Box::new(on_gate_state_change));
    }

    pub fn start(mut self) {
        // Start getting updates from Telegram lib
        self.start_getting_telegram_updates();

        // Start getting gate events
        self.start_getting_gate_state_changes();

        let receiver = self.internal_events_receiver.take()
            .expect("Someone already took the event receiver, this should never happen");

        loop {
            match receiver.recv().expect("Channel was closed! This should never happen!") {
                Event::TelegramMessage(msg_vec) => {
                    for update in msg_vec {
                        self.handle_update(update);
                    }
                },
                Event::GateStateChange(new_state) => {
//                    match new_state {
//                        NewGateState::OPEN => {
//                            self.handle_gate_open();
//                        },
//                        NewGateState::CLOSED => {
//                            self.hardware.turn_off_spotlight();
//                            self.instant_when_was_opened.take();
//                            let should_send_warning = self.last_opening_by_bot.as_ref()
//                                .and_then(|data| Some(data.sent_open_warning && !data.sent_was_closed_info))
//                                .unwrap_or(false);
//                            if should_send_warning {
//                                self.telegram_api.send_msg(OutgoingMessage::new(CHAT_TO_SEND_MSG, "O portão foi fechado!"));
//                            }
//                            if let Some(data) = self.last_opening_by_bot.as_mut(){
//                                data.sent_was_closed_info = true;
//                            }
//                        },
//                    }
                },
                Event::VerifyOpenTooLong => {
//                    let timer_thread_sender = sender.clone();
//                    self.check_gate_open_too_long(timer_thread_sender);
                }
            }
        }
    }
}

