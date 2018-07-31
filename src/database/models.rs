use super::schema::*;

#[derive(Queryable, Clone, AsChangeset)]
#[table_name = "coffeezera_users"]
#[changeset_options(treat_none_as_null = "true")]
pub struct CoffeezeraUser {
    pub id: i32,
    pub name: String,
    pub telegram_id: i64,
    pub account_balance: f64,
    pub is_resident: bool,
}
