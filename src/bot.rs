use crate::bot::RequestedAction::{ConfirmOpen, PrepareOpen, TurnOnLight, Unclassified};
use crate::database::DbUser;
use crate::hardware::RawHardware;
use crate::telegram::{
    ButtonAnswer, Content, DeletableOutgoingMessage, OutgoingMessage, TelegramResponse,
    TelegramUpdate,
};
use crate::State;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};

#[derive(Debug)]
pub struct UserRequest {
    user_id: u64,
    action: RequestedAction,
}

#[derive(Debug)]
enum RequestedAction {
    TurnOnLight { callback_id: String },
    PrepareOpen { callback_id: String },
    ConfirmOpen { callback_id: String },
    Unclassified,
}

impl RequestedAction {
    fn default_buttons() -> Vec<String> {
        vec!["Abrir".to_string(), "Luz 3min".to_string()]
    }
    fn confirm_open_button() -> Vec<String> {
        vec!["Confirmar Abrir".to_string()]
    }
    fn from_button_text(button: &str, callback_query_id: String) -> RequestedAction {
        match button {
            "Luz 3min" => TurnOnLight {
                callback_id: callback_query_id,
            },
            "Abrir" => PrepareOpen {
                callback_id: callback_query_id,
            },
            "Confirmar Abrir" => ConfirmOpen {
                callback_id: callback_query_id,
            },
            x => {
                warn!("Got weird action: {x}");
                Unclassified
            }
        }
    }
}

pub async fn handle_update<T: RawHardware>(
    state: Arc<State<T>>,
    update: TelegramUpdate,
) -> Option<TelegramResponse> {
    // authorize user
    let user_id = i64::try_from(update.user_id).ok()?;
    let user = match state.db.get_user(user_id).await {
        Ok(user) => user,
        Err(e) => {
            error!("{}", e);
            return None;
        }
    };
    let authorized_user = match user {
        None => return Some(unauthorized(user_id)),
        Some(authorized_user) => authorized_user,
    };

    let user_request = parse_user_request(update);
    handle_user_request(state, user_request, authorized_user).await
}

pub async fn handle_user_request<T: RawHardware>(
    state: Arc<State<T>>,
    user_request: UserRequest,
    authorized_user: DbUser,
) -> Option<TelegramResponse> {
    let user_id = i64::try_from(user_request.user_id).expect("Error during id conversion");
    info!(
        "Authorized User {} for request {:?}",
        authorized_user.name, user_request
    );
    let response = match user_request.action {
        Unclassified => Some(default_message(user_id)),
        TurnOnLight { callback_id } => {
            if state.hw.is_spotlight_on().await {
                Some(light_already_on_response(callback_id))
            } else {
                state
                    .hw
                    .turn_on_spotlight(Duration::from_secs(60 * 3))
                    .await;
                Some(light_turned_on_response(callback_id))
            }
        }
        PrepareOpen { callback_id } => {
            state
                .open_requests_waiting_confirmation
                .write()
                .await
                .insert(user_id, std::time::Instant::now());
            Some(prepare_open_message(user_id, callback_id))
        }
        ConfirmOpen { callback_id } => {
            let read_guard = state.open_requests_waiting_confirmation.read().await;
            if let Some(original_request_instant) = read_guard.get(&user_id) {
                if original_request_instant.elapsed().as_secs() < 5 {
                    state.hw.unlock_gate().await;
                    return Some(gate_unlocked_message(callback_id));
                }
            }
            Some(default_message(user_id))
        }
    };
    response
}
fn parse_user_request(update: TelegramUpdate) -> UserRequest {
    let user_request = match update.content {
        Content::Message(_msg) => Unclassified,
        Content::Button {
            text,
            callback_query_id,
        } => RequestedAction::from_button_text(&text, callback_query_id),
    };
    UserRequest {
        user_id: update.user_id,
        action: user_request,
    }
}

fn prepare_open_message(user_id: i64, callback_query_id: String) -> TelegramResponse {
    TelegramResponse::DeletableOutgoingMessage(DeletableOutgoingMessage {
        outgoing_msg: OutgoingMessage {
            user_id,
            message: "Você realmente quer abrir? Essa messagem vai desaparecer em 5s".to_string(),
            buttons: Some(RequestedAction::confirm_open_button()),
        },
        button_answer: Some(ButtonAnswer {
            callback_query_id,
            message: "Veja quem está na porta antes de abrir!".to_string(),
        }),
        delete_after: Some(Duration::new(5, 0)),
    })
}

fn gate_unlocked_message(callback_query: String) -> TelegramResponse {
    TelegramResponse::ButtonAnswer(ButtonAnswer {
        callback_query_id: callback_query,
        message: "Aberto!".to_string(),
    })
}

fn light_turned_on_response(callback_query: String) -> TelegramResponse {
    TelegramResponse::ButtonAnswer(ButtonAnswer {
        callback_query_id: callback_query,
        message: "Luz Ligada".to_string(),
    })
}

fn light_already_on_response(callback_query: String) -> TelegramResponse {
    TelegramResponse::ButtonAnswer(ButtonAnswer {
        callback_query_id: callback_query,
        message: "Já está ligada".to_string(),
    })
}

fn default_message(user_id: i64) -> TelegramResponse {
    TelegramResponse::DeletableOutgoingMessage(DeletableOutgoingMessage {
        outgoing_msg: OutgoingMessage {
            user_id,
            message: "Veja quem está na porta antes de abrir!".to_string(),
            buttons: Some(RequestedAction::default_buttons()),
        },
        delete_after: None,
        button_answer: None,
    })
}

fn unauthorized(user_id: i64) -> TelegramResponse {
    TelegramResponse::DeletableOutgoingMessage(DeletableOutgoingMessage {
        outgoing_msg: OutgoingMessage {
            user_id,
            message: format!("Você não está cadastrado. Envie essa mensagem para @TiberioFerreira com seu id: {}", user_id),
            buttons: None,
        },
        button_answer: None,
        delete_after: None
    })
}
