use diesel::prelude::{Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use crate::schema::{emails, recipients, users};

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = emails)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Email {
    pub id: i32,
    pub sender: String,
}

#[derive(Insertable)]
#[diesel(table_name = emails)]
pub struct NewEmail {
    pub sender: String,
    pub subject: Option<String>,
    pub body: String,
}

#[derive(Insertable)]
#[diesel(table_name = recipients)]
pub struct NewRecipient {
    pub email_id: i32,
    pub recipient: String,
}

#[derive(Queryable, Selectable, Debug, Serialize)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct User {
    id: i32,
    pub email: String,
    pub password_hash: String,
    created_at: chrono::NaiveDateTime,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub email: String,
    pub password_hash: String,
}
