use crate::database::*;
use teleborg::objects::Update;
use teleborg::*;
use super::*;
use std;
use std::time::Instant;
use failure::_core::time::Duration;

const OPEN: &str  = "Abrir";
const TAKE_PIC: &str  = "Tirar Foto";
const YES_OPEN: &str  = "Sim, quero abrir";
const TIME_ALLOWED_FOR_CONFIRMATION: u64 = 5;


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

    fn send_open_confirmation_msg_blocking(&self, chat_id: i64) -> Result<i64, ()>{
        let mut message = OutgoingMessage::new(chat_id, &format!("Tem certeza que deseja abrir? Esta msg é válida por {}s", TIME_ALLOWED_FOR_CONFIRMATION));
        message.with_reply_markup(vec![vec![YES_OPEN.to_string()]]);
        let msg: Message  = self.telegram_api.send_msg_blocking(message)?;
        Ok(msg.message_id)
    }

    fn send_not_registered_msg(&self, chat_id: i64, sender_id: i64){
        let message = OutgoingMessage::new(chat_id, &format!("Você não está registrado. Envie essa mensagem com seu id: {} para @TiberioFerreira.", sender_id));
        self.telegram_api.send_msg(message);
    }

    fn delete_msg_after(&self, chat_id: i64, msg_id: i64, seconds: u64){
        let message = OutgoingDelete{
            chat_id,
            message_id: msg_id
        };
        let sender: Sender<Event> = self.internal_events_sender.clone();
        std::thread::spawn(move||{
            std::thread::sleep(Duration::from_secs(seconds as u64));
            let delete_data = super::OutgoingDelete {
                chat_id,
                message_id: msg_id
            };
            sender.send(Event::DeleteMsg(delete_data)).unwrap();
        });

    }

    fn handle_msg(&self, cleaned_msg: CleanedMessage){
        self.send_default_msg(cleaned_msg.chat_id);
    }

    fn handle_callback(&mut self, cleaned_callback_query: CleanedCallbackQuery, db_user: CoffeezeraUser){
        match cleaned_callback_query.data.as_ref() {
            OPEN => {
                match self.send_open_confirmation_msg_blocking(cleaned_callback_query.original_msg_chat_id){
                    Ok(msg_id) => {
                        self.delete_msg_after(cleaned_callback_query.original_msg_chat_id, msg_id, TIME_ALLOWED_FOR_CONFIRMATION);
                        let people: &mut HashMap<i64, std::time::Instant> = &mut self.last_open_request_without_confirmation;
                        people.insert(cleaned_callback_query.sender_id, std::time::Instant::now());
                        info!("{:?}", people);
                    },
                    Err(()) => {
                        error!("Error sending confirmation msg");
                        return;
                    }
                }

//                self.hardware.unlock_gate();
//                self.last_person_opened = Some(LastPersonOpened {
//                    who_last_opened_it: db_user,
//                    when_user_opened: Instant::now(),
//                    sent_open_warning: false,
//                });
//                self.telegram_api.send_callback_answer(AnswerCallbackQuery{
//                    callback_query_id: cleaned_callback_query.callback_id,
//                    text: Some("Aberto".to_string()),
//                    show_alert: Some(false)
//                });
            },
            YES_OPEN => {
                let people: &mut HashMap<i64, std::time::Instant> = &mut self.last_open_request_without_confirmation;
                people.retain(|telegram_id, instant|{
                    info!("Removing telegram_id: {:?} inserted at {:?}", telegram_id, instant);
                   return instant.elapsed().as_secs() < TIME_ALLOWED_FOR_CONFIRMATION
                });
                if people.get(&cleaned_callback_query.sender_id).is_none(){
                    let delete_data = super::OutgoingDelete {
                        chat_id: cleaned_callback_query.original_msg_chat_id,
                        message_id: cleaned_callback_query.original_msg_id
                    };
                    self.internal_events_sender.send(Event::DeleteMsg(delete_data)).unwrap();
                    return;
                }
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
                let file_path;
                self.telegram_api.send_callback_answer(AnswerCallbackQuery{
                    callback_query_id: cleaned_callback_query.callback_id.clone(),
                    text: Some("Tirando foto...".to_string()),
                    show_alert: Some(false)
                });
                if self.picture_context.last_pic_date.elapsed().as_secs() > 3 {
                    file_path = match self.hardware.take_pic() {
                        Ok(file_path) => file_path,
                        Err(e) => {
                            error!("Error getting photo: {}", e);
                            return;
                        }
                    };
                    self.picture_context.last_pic_date = std::time::Instant::now();
                    self.picture_context.last_pic_path = file_path.clone();
                }else{
                    file_path = self.picture_context.last_pic_path.clone();
                }
                self.telegram_api.send_msg(OutgoingMessage::new(
                    cleaned_callback_query.original_msg_chat_id.clone(),
                "Enviando Foto.."));
                self.telegram_api.send_photo(OutgoingPhoto::new(cleaned_callback_query.original_msg_chat_id, &file_path));
                self.send_default_msg(cleaned_callback_query.original_msg_chat_id);
            },
            e => {
                error!("Unexpected Callback: {}", e);
            }
        }

    }
}