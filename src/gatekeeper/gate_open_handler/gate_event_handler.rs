use super::*;
use std::time::*;
use std::thread;
use chrono::prelude::*;
// pub const CHAT_TO_SEND_MSG: i64 = 75698394; // Tiberio
pub const CHAT_TO_SEND_MSG: i64 = -1001121845452; // Rep
const NB_SEC_OPEN_TOO_LONG: u64 = 5*60;



impl <T: TelegramInterface> GringosGateKeeperBot<T> {

    pub (crate) fn handle_gate_state_change(&mut self, change: NewGateState){
        match change{
            NewGateState::OPEN => {
                self.handle_gate_open();
            },
            NewGateState::CLOSED => {
                self.handle_gate_closed();
            },
        }
    }

//    pub (crate) fn who_just_opened_it(&self){
//        self.last_opening_by_bot.and_then(|last_opening| {
//            last_opening.
//        })
//    }

    pub (crate) fn handle_gate_closed(&mut self){
        self.hardware.turn_off_spotlight();
    }
    pub (crate) fn handle_gate_open(&mut self){
        let dt = chrono::Local::now();
        if dt.hour() <= 6 || dt.hour() >= 18 {
            self.hardware.turn_on_spotlight();
            self.last_gate_open_event = Some(LastGateOpenEvent{
                instant_when_was_opened_if_is_open: Instant::now(),
                turned_on_spot_light: true
            });
        }
        

//        if let Some(ref current_opening) = self.current_opening_by_bot {
//
//        }
//        let mut already_sent_msg = false;
//        if let Some(ref last_opening_by_bot_data) = self.last_opening_by_bot{
//            if last_opening_by_bot_data.when_user_opened.elapsed().as_secs() < 10 {
//                self.telegram_api.send_msg(
//                    OutgoingMessage::new(75698394,
//                                         format!("O portão foi aberto por {}!",
//                                                 last_opening_by_bot_data.who_last_opened_it.name)
//                                             .as_str()
//                    )
//                );
//                already_sent_msg = true;
//            }
//        };
//        if !already_sent_msg {
//            self.telegram_api.send_msg(
//                OutgoingMessage::new(75698394,
//                                     "O portão foi aberto usando chave!")
//            );
//        }
//        thread::spawn(move ||{
//            thread::sleep(Duration::from_secs(NB_SEC_OPEN_TOO_LONG));
//            timer_thread_sender.send(Event::VerifyOpenTooLong);
//        });
    }

//    pub (crate) fn check_gate_open_too_long(&mut self, timer_thread_sender: Sender<Event>){
//        info!("Got to verify!");
//        if let Some(time_was_opened) = self.instant_when_was_opened {
//            if time_was_opened.elapsed().as_secs() >= NB_SEC_OPEN_TOO_LONG {
//                if let Some(ref mut last_opening_by_bot_data) = self.current_opening_by_bot {
//                    if last_opening_by_bot_data.when_user_opened <= time_was_opened &&
//                        time_was_opened - last_opening_by_bot_data.when_user_opened < Duration::from_secs(10){
//                        self.telegram_api.send_msg(
//                            OutgoingMessage::new(CHAT_TO_SEND_MSG,
//                                                 format!("O portão está aberto a mais de {} minutos. {} o abriu e não fechou ainda.",
//                                                         time_was_opened.elapsed().as_secs()/60,
//                                                         last_opening_by_bot_data.who_last_opened_it.name).as_str()));
//                    }else{
//                        self.telegram_api.send_msg(
//                            OutgoingMessage::new(CHAT_TO_SEND_MSG,
//                                                 format!("Alguém abriu o portão com chave e já deixou aberto por {} minutos.",
//                                                         time_was_opened.elapsed().as_secs()/60).as_str()));
//                    }
//                    last_opening_by_bot_data.sent_open_warning = true;
//                }
//                if time_was_opened.elapsed().as_secs()/60 > 30 {
//                    self.telegram_api.send_msg(
//                        OutgoingMessage::new(CHAT_TO_SEND_MSG,
//                                             "O portão está aberto a mais de meia hora. Desisto, não vou mais enviar mensagens sobre isso ate ele ser fechado."));
//                    self.hardware.turn_off_spotlight();
//                    self.instant_when_was_opened.take();
//                }else{
//                    thread::spawn(move ||{
//                        thread::sleep(Duration::from_secs(NB_SEC_OPEN_TOO_LONG));
//                        timer_thread_sender.send(Event::VerifyOpenTooLong);
//                    });
//                }
//                info!("Sent warning!");
//            }
//        }
//    }
}