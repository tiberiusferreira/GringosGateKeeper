extern crate failure;
use diesel::{PgConnection};
use database::establish_connection;
use std::env;
use teleborg::*;
mod update_handler;
pub struct GringosGateKeeperBot<T> where T: TelegramInterface{
    telegram_api: T,
    database_connection: PgConnection
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
        }
    }

    pub fn start(&mut self){

        self.telegram_api.start_getting_updates();
        let update_channel = self.telegram_api.get_updates_channel();
        loop {
            let updates = update_channel.recv().unwrap();
            for update in updates {
                self.handle_update(update);
            }
        }
    }
}

