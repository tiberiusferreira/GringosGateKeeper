use diesel::{PgConnection};
use ::database::models::CoffeezeraUser;
use database::*;
use teleborg::objects::Update;
use teleborg::*;
use super::*;

const OPEN: &'static str  = "Abrir";

struct CleanedSender{
    sender_id: i64,
    chat_id: i64,
    db_user: CoffeezeraUser
}

impl CleanedSender{
    pub fn from_message(message: Message, db_con: &PgConnection) -> Result<Self, Option<OutgoingMessage>> {
        let chat_id = message.chat.id;
        let sender_id = match message.from {
            Some(sender) => sender.id,
            None => {
                return Err(None)
            }
        };
        let db_user = match get_user(db_con, sender_id) {
            Ok(coffeezera_user) => coffeezera_user,
            Err(_) => {
                return Err(Some(OutgoingMessage::new(chat_id, &format!("Você não está registrado. Envie essa mensagem para @TiberioFerreira com seu ID: {}", sender_id))));
            }
        };
        Ok(CleanedSender{
            sender_id,
            chat_id,
            db_user
        })
    }
    pub fn from_callback_query(callback: CallBackQuery, db_con: &PgConnection) -> Result<Self, Option<OutgoingMessage>> {
        let chat_id = match callback.message {
            Some(msg) => msg.chat.id,
            None => {
                error!("Callback without Message!");
                return Err(None);
            }
        };
        let sender_id = callback.from.id;
        let db_user = match get_user(db_con, sender_id) {
            Ok(coffeezera_user) => coffeezera_user,
            Err(_) => {
                return Err(Some(OutgoingMessage::new(chat_id, &format!("Você não está registrado. Envie essa mensagem para @TiberioFerreira com seu ID: {}", sender_id))));
            }
        };
        Ok(CleanedSender{
            sender_id,
            chat_id,
            db_user
        })
    }
}

struct CleanedCallbackQuery{
    sender: CleanedSender,
    id: String,
    data: String
}

impl CleanedCallbackQuery{
    pub fn new(callback: CallBackQuery, db_con: &PgConnection) -> Result<Self, Option<OutgoingMessage>>{
        let sender = CleanedSender::from_callback_query(callback.clone(), db_con)?;
        let data = match callback.data{
            Some(data) => data,
            None => {
                return Err(Some(OutgoingMessage::new(sender.sender_id, "Callback sem data, envie isso para @TiberioFerreira")));
            }
        };
        Ok(CleanedCallbackQuery{
            sender,
            id: callback.id,
            data
        })
    }
}
impl <T: TelegramInterface> GringosGateKeeperBot<T> {
    pub (in super) fn handle_update(&self, update: Update){
        info!("Got update: {:?}", update);
        if let Some(message) = update.message {
            let cleaned_sender = match CleanedSender::from_message(message, &self.database_connection){
                Ok(cleaned_sender) => cleaned_sender,
                Err(maybe_outgoing_message) => {
                    if let Some(msg) = maybe_outgoing_message{
                        self.telegram_api.send_msg(msg);
                    }
                    return;
                }
            };
            info!("This was a message update");
            self.handle_msg(cleaned_sender);
            return;
        }
        if let Some(callback_query) = update.callback_query {
            let cleaned_callback = match CleanedCallbackQuery::new(callback_query, &self.database_connection){
                Ok(cleaned_callback) => cleaned_callback,
                Err(maybe_outgoing_message) => {
                    if let Some(msg) = maybe_outgoing_message{
                        self.telegram_api.send_msg(msg);
                    }
                    return;
                }
            };
            info!("This was a callback update");
            self.handle_callback(cleaned_callback);
            return;
        }
        error!("This was neither a Message or Callback update. Weird.");
    }

    fn handle_msg(&self, cleaned_msg_sender: CleanedSender){
        let mut message = OutgoingMessage::new(cleaned_msg_sender.chat_id, "Veja quem está na porta antes de abrir!");
        message.with_reply_markup(vec![vec![OPEN.to_string()]]);
        self.telegram_api.send_msg(message);
    }

    fn handle_callback(&self, callback_query: CleanedCallbackQuery){
        match callback_query.data.as_ref() {
            OPEN => {
                self.telegram_api.send_callback_answer(AnswerCallbackQuery{
                    callback_query_id: callback_query.id,
                    text: Some("Aberto".to_string()),
                    show_alert: Some(false)
                });
            },
            e => {
                error!("Unexpected Callback: {}", e);
            }
        }

    }
}