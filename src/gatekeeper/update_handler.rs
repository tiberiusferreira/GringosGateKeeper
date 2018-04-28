use database::*;
use teleborg::objects::Update;
use teleborg::*;
use super::*;
use std;
const OPEN: &'static str  = "Abrir";
const TAKE_PIC: &'static str  = "Tirar Foto";

impl <T: TelegramInterface> GringosGateKeeperBot<T> {
    pub (in super) fn handle_update(&mut self, update: Update){
        info!("Got update: {:?}", update);
        match clean_update(update){
            Ok(cleaned_update) => {
                if self.check_user_is_authorized_reply_if_not(cleaned_update.clone()){
                    match cleaned_update {
                        CleanedUpdate::CleanedMessage(cleaned_msg) => self.handle_msg(cleaned_msg),
                        CleanedUpdate::CleanedCallbackQuery(cleaned_callback) => self.handle_callback(cleaned_callback)
                    };
                }
            },
            Err(err) => {
                error!("{}", err);
            }
        }
    }

    fn check_user_is_authorized_reply_if_not(&self, cleaned_update: CleanedUpdate) -> bool{
        let (sender_id, sender_chat_id) = match cleaned_update {
            CleanedUpdate::CleanedMessage(cleaned_msg) => (cleaned_msg.sender_id, cleaned_msg.chat_id),
            CleanedUpdate::CleanedCallbackQuery(cleaned_callback) => (cleaned_callback.sender_id, cleaned_callback.original_msg_chat_id)
        };
        match get_user(&self.database_connection, sender_id){
            Ok(db_user) => {
                if db_user.is_resident{
                    return true;
                }else{
                    self.send_not_resident_reply(sender_chat_id);
                    return false;
                }
            },
            Err(_) =>{
                self.send_not_registered_msg(sender_chat_id, sender_id);
                return false;
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

    fn handle_callback(&mut self, cleaned_callback_query: CleanedCallbackQuery){
        match cleaned_callback_query.data.as_ref() {
            OPEN => {
                self.hardware.open_gate();
                self.telegram_api.send_callback_answer(AnswerCallbackQuery{
                    callback_query_id: cleaned_callback_query.callback_id,
                    text: Some("Aberto".to_string()),
                    show_alert: Some(false)
                });
            },
            TAKE_PIC => {
                let file_path;
                if self.last_pic_date.elapsed().as_secs() > 3 {
                    file_path = match self.hardware.take_pic() {
                        Ok(file_path) => file_path,
                        Err(e) => {
                            error!("Error getting photo: {}", e);
                            self.telegram_api.send_callback_answer(AnswerCallbackQuery {
                                callback_query_id: cleaned_callback_query.callback_id,
                                text: Some("Problema com a camera vagabunda. Fale com @TiberioFerreira".to_string()),
                                show_alert: Some(false)
                            });
                            return;
                        }
                    };
                    self.last_pic_date = std::time::Instant::now();
                    self.last_pic_path = file_path.clone();
                }else{
                    file_path = self.last_pic_path.clone();
                }
                self.telegram_api.send_callback_answer(AnswerCallbackQuery{
                    callback_query_id: cleaned_callback_query.callback_id,
                    text: Some("Enviando foto...".to_string()),
                    show_alert: Some(false)
                });
                self.telegram_api.send_photo(OutgoingPhoto::new(cleaned_callback_query.original_msg_chat_id, &file_path));
            },
            e => {
                error!("Unexpected Callback: {}", e);
            }
        }

    }
}