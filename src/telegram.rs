use async_trait::async_trait;
use std::time::Duration;
use tracing::debug;

mod frankenstein_impl;
pub use frankenstein_impl::FrankensteinWrapper;
#[derive(Debug, Clone)]
pub struct TelegramUpdate {
    pub user_id: u64,
    pub content: Content,
}

#[derive(Debug, Clone)]
pub enum Content {
    Message(String),
    Button {
        text: String,
        callback_query_id: String,
    },
}

#[derive(Debug, Clone)]
pub enum TelegramResponse {
    DeletableOutgoingMessage(DeletableOutgoingMessage),
    ButtonAnswer(ButtonAnswer),
}

#[derive(Debug, Clone)]
pub struct OutgoingMessage {
    pub user_id: i64,
    pub message: String,
    pub buttons: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct ButtonAnswer {
    pub callback_query_id: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct DeletableOutgoingMessage {
    pub outgoing_msg: OutgoingMessage,
    pub delete_after: Option<Duration>,
}

#[async_trait]
pub trait TelegramInterface {
    fn start_getting_updates(&mut self) -> tokio::sync::mpsc::Receiver<Vec<TelegramUpdate>>;
    async fn send_response(&self, response: TelegramResponse) -> Result<(), String> {
        match response {
            TelegramResponse::DeletableOutgoingMessage(msg) => {
                self.send_message_and_sleep_then_delete_if_needed(msg).await
            }
            TelegramResponse::ButtonAnswer(button_answer) => {
                self.send_button_answer(button_answer).await
            }
        }
    }
    async fn send_button_answer(&self, msg: ButtonAnswer) -> Result<(), String>;
    async fn send_message(&self, msg: OutgoingMessage) -> Result<i32, String>;
    async fn delete_message(&self, user_id: i64, message_id: i32) -> Result<(), String>;
    async fn send_message_and_sleep_then_delete_if_needed(
        &self,
        msg: DeletableOutgoingMessage,
    ) -> Result<(), String> {
        let msg_id = self.send_message(msg.outgoing_msg.clone()).await?;
        if let Some(deleted_after) = msg.delete_after {
            tokio::time::sleep(deleted_after).await;
            debug!(
                "Deleting original msg with id: {} from chat {}",
                msg_id, msg.outgoing_msg.user_id
            );
            self.delete_message(msg.outgoing_msg.user_id, msg_id)
                .await?;
        };
        Ok(())
    }
}
