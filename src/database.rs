use sqlx::PgPool;

pub struct Database {
    con: PgPool,
}

pub struct DbUser {
    pub name: String,
}
impl Database {
    pub fn new(con: PgPool) -> Database {
        Self { con }
    }
    pub async fn get_user(&self, telegram_id: i64) -> Result<Option<DbUser>, String> {
        let user: Option<DbUser> = sqlx::query_as!(
            DbUser,
            "select name from users where telegram_id = $1 ;",
            telegram_id
        )
        .fetch_optional(&self.con)
        .await
        .map_err(|e| e.to_string())?;
        Ok(user)
    }
}
