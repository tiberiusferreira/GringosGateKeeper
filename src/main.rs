mod bot;
mod database;
mod hardware;

use crate::bot::handle_update;
use crate::database::Database;
use crate::hardware::{RawHardware, RealHardware, RefCountedGateHardware};
use crate::telegram::{FrankensteinWrapper, TelegramInterface};
use sqlx::postgres::PgPoolOptions;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::error;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt::time::OffsetTime, EnvFilter};

mod telegram;

fn start_logging() -> WorkerGuard {
    let offset = time::UtcOffset::from_hms(3, 0, 0).expect("offset should work");
    let time_format =
        time::format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]")
            .expect("format string should be valid!");
    let timer = OffsetTime::new(offset, time_format);
    let file_appender = RollingFileAppender::new(Rotation::DAILY, "./logs", "log");
    let (non_blocking, logs_flush_guard) = tracing_appender::non_blocking(file_appender);
    let subscriber = tracing_subscriber::fmt()
        .with_timer(timer)
        .with_ansi(false)
        .with_writer(non_blocking)
        .with_env_filter(EnvFilter::new("gate=debug"))
        .with_filter_reloading();
    subscriber.init();
    logs_flush_guard
}

pub struct State<T: RawHardware> {
    pub hw: RefCountedGateHardware<T>,
    pub open_requests_waiting_confirmation: RwLock<HashMap<i64, Instant>>,
    pub db: Database,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let _guard = start_logging();
    let old_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        error!("{}", panic.to_string());
        old_hook(panic);
    }));
    let pg_db_pool = PgPoolOptions::new()
        .max_connections(3)
        .connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .expect("Error connecting to the DB");
    let db = Database::new(pg_db_pool);
    let mut fk = FrankensteinWrapper::new();
    let mut receiver = fk.start_getting_updates();
    let state = Arc::new(State {
        hw: RefCountedGateHardware::<RealHardware>::new_real_hardware(),
        open_requests_waiting_confirmation: RwLock::new(HashMap::new()),
        db,
    });
    loop {
        let updates = receiver.recv().await.expect("telegram disconnected!");
        for update in updates {
            let maybe_response = handle_update(Arc::clone(&state), update).await;
            if let Some(response) = maybe_response {
                let fk_clone = fk.clone();
                tokio::spawn(async move { fk_clone.send_response(response).await });
            }
        }
    }
}
