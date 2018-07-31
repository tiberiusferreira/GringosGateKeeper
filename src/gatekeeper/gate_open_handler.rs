use teleborg::TelegramInterface;
use gatekeeper::GringosGateKeeperBot;
use teleborg::objects::OutgoingMessage;
use std::time::Instant;
extern crate chrono;

use self::chrono::prelude::*;

impl <T: TelegramInterface> GringosGateKeeperBot<T> {
    pub fn handle_gate_open(&mut self){
        let dt = chrono::Local::now();
        if dt.hour() <= 7 || dt.hour() >= 17 {
            self.hardware.turn_on_spotlight();
        }
        if let Some(time_was_opened) = self.instant_when_was_opened{
            if time_was_opened.elapsed().as_secs() > 5*60 && self.sent_open_warning==false{
                self.telegram_api.send_msg(OutgoingMessage::new(75698394, "O portão está aberto a mais de 5 minutos!"));
                self.instant_when_was_opened.take();
                self.sent_open_warning = true;
            }
        }else{
            self.instant_when_was_opened = Some(Instant::now());
            self.telegram_api.send_msg(OutgoingMessage::new(75698394, "O portão foi aberto!"));
            self.sent_open_warning = false;
        }
    }
}