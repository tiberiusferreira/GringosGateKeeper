extern crate failure;
mod update_handler;
mod gate_open_handler;

use diesel::{PgConnection};
use database::establish_connection;
use std;
use std::env;
use teleborg::*;
use super::hardware::*;
pub struct GringosGateKeeperBot<T> where T: TelegramInterface{
    telegram_api: T,
    database_connection: PgConnection,
    hardware: Hardware,
    last_pic_date: std::time::Instant,
    last_pic_path: String,
    instant_when_was_opened: Option<std::time::Instant>
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
            instant_when_was_opened: None
        }
    }

    pub fn emergency_turn_off(){
        let hw = Hardware::new();
        hw.turn_off_spotlight();
        hw.close_gate();
    }

    pub fn start(&mut self){

        self.telegram_api.start_getting_updates();

        loop {
            if let Ok(updates) = self.telegram_api.get_updates_channel().try_recv(){
                for update in updates {
                    self.handle_update(update);
                }
            }
            if self.hardware.gate_is_open(){
                self.handle_gate_open();
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    }
}

