use crate::telegram::{ButtonAnswer, Content, OutgoingMessage, TelegramInterface, TelegramUpdate};
use async_trait::async_trait;
use frankenstein::{
    AllowedUpdate, AnswerCallbackQueryParams, AsyncApi, AsyncTelegramApi, CallbackQuery, ChatId,
    DeleteMessageParams, InlineKeyboardButton, InlineKeyboardMarkup, Message, ReplyMarkup,
    SendMessageParams, Update, UpdateContent,
};
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;

use tracing::{debug, error, info};

#[derive(Clone)]
pub struct FrankensteinWrapper {
    telegram: Arc<AsyncApi>,
}

impl FrankensteinWrapper {
    pub fn new() -> Self {
        let token = std::env::var("BOT_API_TOKEN").expect("No bot api token");
        let api = AsyncApi::new(&token);
        Self {
            telegram: Arc::new(api),
        }
    }
}

struct FrankensteinReceiverWrapper {
    telegram: Arc<AsyncApi>,
    last_processed_update_id: i64,
}

impl FrankensteinReceiverWrapper {
    pub async fn get_updates(&mut self) -> Result<Vec<TelegramUpdate>, String> {
        let current_update_id = self.last_processed_update_id;
        let update_params = frankenstein::GetUpdatesParams {
            offset: Some(current_update_id + 1),
            limit: Some(50),
            timeout: Some(30),
            allowed_updates: Some(vec![AllowedUpdate::Message, AllowedUpdate::CallbackQuery]),
        };
        let result: Vec<Update> = self
            .telegram
            .get_updates(&update_params)
            .await
            .map_err(|e| e.to_string())?
            .result;

        let updates: Vec<UpdateContent> = result
            .into_iter()
            .map(|update| {
                if update.update_id as i64 > self.last_processed_update_id {
                    self.last_processed_update_id = update.update_id as i64;
                }
                update.content
            })
            .collect();
        debug!("Raw Updates: {:?}", updates);
        let updates: Vec<TelegramUpdate> = updates
            .into_iter()
            .filter_map(|update| match update {
                UpdateContent::Message(Message {
                    text: Some(message_text),
                    date: msg_unix_timestamp,
                    from: Some(user),
                    ..
                }) => {
                    let oldest_timestamp_to_get_msg_for =
                        u64::try_from(chrono::Utc::now().timestamp() - 10)
                            .expect("weird timestamp");
                    if msg_unix_timestamp < oldest_timestamp_to_get_msg_for {
                        info!(
                            "Discarding older messages: {} from {:?}",
                            message_text, user
                        );
                        None
                    } else {
                        Some(TelegramUpdate {
                            user_id: user.id,
                            content: Content::Message(message_text),
                        })
                    }
                }
                UpdateContent::CallbackQuery(CallbackQuery {
                    id,
                    from: user,
                    data: Some(callback_data),
                    ..
                }) => Some(TelegramUpdate {
                    user_id: user.id,
                    content: Content::Button {
                        text: callback_data,
                        callback_query_id: id,
                    },
                }),
                x => {
                    info!("Ignoring update: {:?}", x);
                    None
                }
            })
            .collect();
        Ok(updates)
    }
}

#[async_trait]
impl TelegramInterface for FrankensteinWrapper {
    fn start_getting_updates(&mut self) -> Receiver<Vec<TelegramUpdate>> {
        let mut telegram_receiver = FrankensteinReceiverWrapper {
            telegram: Arc::clone(&self.telegram),
            last_processed_update_id: 0,
        };
        let (tx, rx) = tokio::sync::mpsc::channel::<Vec<TelegramUpdate>>(10);
        tokio::spawn(async move {
            loop {
                match telegram_receiver.get_updates().await {
                    Ok(updates) => {
                        if !updates.is_empty() {
                            if let Err(e) = tx.try_send(updates) {
                                error!("Sending updates: {e:?}");
                            }
                        }
                    }
                    Err(e) => {
                        error!("Getting updates: {e}");
                        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                        continue;
                    }
                }
            }
        });
        rx
    }

    async fn send_button_answer(&self, msg: ButtonAnswer) -> Result<(), String> {
        self.telegram
            .answer_callback_query(&AnswerCallbackQueryParams {
                callback_query_id: msg.callback_query_id,
                text: Some(msg.message),
                show_alert: None,
                url: None,
                cache_time: Some(3),
            })
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn send_message(&self, msg: OutgoingMessage) -> Result<i32, String> {
        let buttons = msg.buttons.map(|buttons| {
            let buttons: Vec<InlineKeyboardButton> = buttons
                .into_iter()
                .map(|single_button| InlineKeyboardButton {
                    text: single_button.clone(),
                    url: None,
                    login_url: None,
                    callback_data: Some(single_button),
                    web_app: None,
                    switch_inline_query: None,
                    switch_inline_query_current_chat: None,
                    callback_game: None,
                    pay: None,
                })
                .collect();
            ReplyMarkup::InlineKeyboardMarkup(InlineKeyboardMarkup {
                inline_keyboard: vec![buttons],
            })
        });
        let msg = SendMessageParams {
            chat_id: ChatId::Integer(msg.user_id),
            message_thread_id: None,
            text: msg.message,
            parse_mode: None,
            entities: None,
            disable_web_page_preview: None,
            disable_notification: None,
            protect_content: None,
            reply_to_message_id: None,
            allow_sending_without_reply: None,
            reply_markup: buttons,
        };
        let msg_id = self
            .telegram
            .send_message(&msg)
            .await
            .map_err(|e| e.to_string())?
            .result
            .message_id;
        Ok(msg_id)
    }

    async fn delete_message(&self, user_id: i64, message_id: i32) -> Result<(), String> {
        self.telegram
            .delete_message(&DeleteMessageParams {
                chat_id: ChatId::Integer(user_id),
                message_id,
            })
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
