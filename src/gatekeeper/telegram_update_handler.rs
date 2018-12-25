use crate::database::*;
use teleborg::objects::Update;
use teleborg::*;
use super::*;
use std;
use std::time::Instant;

const OPEN: &str  = "Abrir";
const TAKE_PIC: &str  = "Tirar Foto";



impl <T: TelegramInterface> GringosGateKeeperBot<T> {

    pub (in super) fn start_getting_telegram_updates(&mut self){
        // Map Telegram updates to our own Event type, this needs to be non-blocking since
        // we need to check other events too
        // TODO make Teleborg take a function, so we pass this to him and remove this thread
        self.telegram_api.start_getting_updates();
        let telegram_update_receiver = self.telegram_api.get_updates_channel().clone();
        let telegram_thread_sender = self.internal_events_sender.clone();
        std::thread::spawn(move || {
            while let Ok(update) = telegram_update_receiver.recv() {
                telegram_thread_sender.send(Event::TelegramMessage(update)).expect("Failed to send Telegram Event");
            }
        });
    }

    pub (in super) fn handle_update(&mut self, update: Update){
        info!("Got update: {:?}", update);
        match clean_update(update){
            Ok(cleaned_update) => {
                if let Some(db_user) = self.get_user_if_authorized_reply_if_not(cleaned_update.clone()) {
                    match cleaned_update {
                        CleanedUpdate::CleanedMessage(cleaned_msg) => self.handle_msg(cleaned_msg),
                        CleanedUpdate::CleanedCallbackQuery(cleaned_callback) => self.handle_callback(cleaned_callback, db_user)
                    };
                }
            },
            Err(err) => {
                error!("{}", err);
            }
        }
    }

    fn get_user_if_authorized_reply_if_not(&self, cleaned_update: CleanedUpdate) -> Option<CoffeezeraUser>{
        let (sender_id, sender_chat_id) = match cleaned_update {
            CleanedUpdate::CleanedMessage(cleaned_msg) => (cleaned_msg.sender_id, cleaned_msg.chat_id),
            CleanedUpdate::CleanedCallbackQuery(cleaned_callback) => (cleaned_callback.sender_id, cleaned_callback.original_msg_chat_id)
        };
        match get_user(&self.database_connection, sender_id){
            Ok(db_user) => {
                if db_user.is_resident{
                    return Some(db_user);
                }else{
                    self.send_not_resident_reply(sender_chat_id);
                    return None;
                }
            },
            Err(_) =>{
                self.send_not_registered_msg(sender_chat_id, sender_id);
                return None;
            }
        }
    }

    fn send_not_resident_reply(&self, chat_id: i64){
        let message = OutgoingMessage::new(chat_id, "Você não é morador, logo não pode abrir o portão. :(");
        self.telegram_api.send_msg(message);
    }

    fn send_default_msg(&self, chat_id: i64){
        let mut message = OutgoingMessage::new(chat_id, "Veja quem está na porta antes de abrir!");
        message.with_reply_markup(vec![vec![OPEN.to_string(), TAKE_PIC.to_string()]]);
        self.telegram_api.send_msg(message);
    }

    fn send_not_registered_msg(&self, chat_id: i64, sender_id: i64){
        let message = OutgoingMessage::new(chat_id, &format!("Você não está registrado. Envie essa mensagem com seu id: {} para @TiberioFerreira.", sender_id));
        self.telegram_api.send_msg(message);
    }

    fn handle_msg(&self, cleaned_msg: CleanedMessage){
        self.send_default_msg(cleaned_msg.chat_id);
    }

    fn handle_callback(&mut self, cleaned_callback_query: CleanedCallbackQuery, db_user: CoffeezeraUser){
        match cleaned_callback_query.data.as_ref() {
            OPEN => {
                self.hardware.unlock_gate();
                self.last_person_opened = Some(LastPersonOpened {
                    who_last_opened_it: db_user,
                    when_user_opened: Instant::now(),
                    sent_open_warning: false,
                });
                self.telegram_api.send_callback_answer(AnswerCallbackQuery{
                    callback_query_id: cleaned_callback_query.callback_id,
                    text: Some("Aberto".to_string()),
                    show_alert: Some(false)
                });
            },
            TAKE_PIC => {
//                let file_path;
//                if self.picture_context.last_pic_date.elapsed().as_secs() > 10 {
//                    file_path = match self.hardware.take_pic() {
//                        Ok(file_path) => file_path,
//                        Err(e) => {
//                            error!("Error getting photo: {}", e);
//                            self.telegram_api.send_callback_answer(AnswerCallbackQuery {
//                                callback_query_id: cleaned_callback_query.callback_id,
//                                text: Some("Problema com a camera vagabunda. Fale com @TiberioFerreira".to_string()),
//                                show_alert: Some(false)
//                            });
//                            return;
//                        }
//                    };
//                    self.picture_context.last_pic_date = std::time::Instant::now();
//                    self.picture_context.last_pic_path = file_path.clone();
//                }else{
//                    file_path = self.picture_context.last_pic_path.clone();
//                }
//                self.telegram_api.send_callback_answer(AnswerCallbackQuery{
//                    callback_query_id: cleaned_callback_query.callback_id,
//                    text: Some("Enviando foto...".to_string()),
//                    show_alert: Some(false)
//                });
//                self.telegram_api.send_photo(OutgoingPhoto::new(cleaned_callback_query.original_msg_chat_id, &file_path));
//                self.send_default_msg(cleaned_callback_query.original_msg_chat_id);
            },
            e => {
                error!("Unexpected Callback: {}", e);
            }
        }

    }
}