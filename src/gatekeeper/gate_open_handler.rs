use teleborg::TelegramInterface;
use gatekeeper::GringosGateKeeperBot;
use teleborg::objects::OutgoingMessage;
use std::time::Instant;

impl <T: TelegramInterface> GringosGateKeeperBot<T> {
    pub fn handle_gate_open(&mut self){
        if let Some(time_was_opened) = self.instant_when_was_opened{
            if time_was_opened.elapsed().as_secs() > 300{
                self.telegram_api.send_msg(OutgoingMessage::new(75698394, "O portão está aberto a mais de 5 minutos!"));
                self.instant_when_was_opened.take();
            }
        }else{
            self.instant_when_was_opened = Some(Instant::now());
        }
    }
}