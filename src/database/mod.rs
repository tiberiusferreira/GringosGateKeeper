pub mod schema;
pub mod models;

extern crate dotenv;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use self::dotenv::dotenv;
use std::env;
use diesel::result;



pub fn establish_connection() -> PgConnection {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

use self::models::CoffeezeraUser;


pub fn get_user<'a>(conn: &PgConnection, input_telegram_id: i64) -> Result<CoffeezeraUser, result::Error> {
    use self::schema::coffeezera_users::dsl::{coffeezera_users, telegram_id};

    coffeezera_users.filter(telegram_id.eq(input_telegram_id))
        .get_result::<CoffeezeraUser>(conn)
}