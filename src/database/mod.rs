pub mod schema;
pub mod models;

extern crate dotenv;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use std::env;
use diesel::result;
use self::models::CoffeezeraUser;
use self::schema::coffeezera_users::dsl::*;
use self::dotenv::dotenv;
use std::time::Duration;
use std;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let mut number_errors_up_to_2000 = 0;
    loop {
        match PgConnection::establish(&database_url) {
            Ok(pg_connection) => return pg_connection,
            Err(e) => {
                number_errors_up_to_2000 = (number_errors_up_to_2000 + 1) % 2000;
                error!("Error connecting to DB: {:?}", e);
                error!("Sleeping for: {} seconds.", 60*number_errors_up_to_2000);
                std::thread::sleep(Duration::from_secs(60*number_errors_up_to_2000));
            }
        }
    }

}


pub fn get_user<'a>(conn: &PgConnection, input_telegram_id: i64) -> Result<CoffeezeraUser, result::Error> {
    coffeezera_users.filter(telegram_id.eq(input_telegram_id))
        .get_result::<CoffeezeraUser>(conn)
}