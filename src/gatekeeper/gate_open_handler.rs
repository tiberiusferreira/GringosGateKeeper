use teleborg::TelegramInterface;
use gatekeeper::GringosGateKeeperBot;
use teleborg::objects::OutgoingMessage;
use std::time::Instant;
extern crate chrono;

use self::chrono::prelude::*;
use crossbeam_channel::Sender;
use gatekeeper::Event;
use std::{thread, time::{Duration}};

const NB_SEC_OPEN_TOO_LONG: u64 = 5*60;
impl <T: TelegramInterface> GringosGateKeeperBot<T> {
    pub (crate) fn handle_gate_open(&mut self, timer_thread_sender: Sender<Event>){
        let dt = chrono::Local::now();
        if dt.hour() <= 7 || dt.hour() >= 17 {
            self.hardware.turn_on_spotlight();
        }
        self.instant_when_was_opened = Some(Instant::now());
        let mut already_sent_msg = false;
        if let Some(ref last_opening_by_bot_data) = self.last_opening_by_bot{
            if last_opening_by_bot_data.when_user_opened.elapsed().as_secs() < 10 {
                self.telegram_api.send_msg(
                    OutgoingMessage::new(75698394,
                                         format!("O portão foi aberto por {}!",
                                                 last_opening_by_bot_data.who_last_opened_it.name)
                                             .as_str()
                    )
                );
                already_sent_msg = true;
            }
        };
        if !already_sent_msg {
            self.telegram_api.send_msg(
                OutgoingMessage::new(75698394,
                                     "O portão foi aberto usando chave!")
            );
        }
        thread::spawn(move ||{
            thread::sleep(Duration::from_secs(NB_SEC_OPEN_TOO_LONG));
            timer_thread_sender.send(Event::VerifyOpenTooLong);
        });
    }

    pub (crate) fn check_gate_open_too_long(&mut self){
        info!("Got to verify!");
        if let Some(time_was_opened) = self.instant_when_was_opened {
            if time_was_opened.elapsed().as_secs() >= NB_SEC_OPEN_TOO_LONG {
                if let Some(ref last_opening_by_bot_data) = self.last_opening_by_bot{
                    if last_opening_by_bot_data.when_user_opened > time_was_opened &&
                        last_opening_by_bot_data.when_user_opened - time_was_opened > Duration::from_secs(10){
                        self.telegram_api.send_msg(
                            OutgoingMessage::new(75698394,
                                                 format!("O portão está aberto a mais de 5 minutos. {} o abriu e não fechou ainda.", last_opening_by_bot_data.who_last_opened_it.name).as_str()));
                    }else{
                        self.telegram_api.send_msg(
                            OutgoingMessage::new(75698394,
                                                 "Alguém abriu o portão com chave e já deixou aberto por 5 minutos."));
                    }
                }
                self.instant_when_was_opened.take();
                info!("Sent warning!");
            }
        }
    }
}