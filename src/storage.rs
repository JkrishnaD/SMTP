use diesel::{
    dsl::exists,
    prelude::*,
    r2d2::{ConnectionManager, Pool},
    sqlite::SqliteConnection,
};

use crate::{models::NewUser, schema::{emails, recipients}};
use crate::{
    models::{Email, NewEmail, NewRecipient, User},
    schema::users,
};

// struct for pooling database connections
#[derive(Clone)]
pub struct Store {
    pub pool: Pool<ConnectionManager<SqliteConnection>>,
}

// methods for interacting with the database
impl Store {
    pub fn new(pool: Pool<ConnectionManager<SqliteConnection>>) -> Self {
        Self { pool }
    }

    pub async fn save_emails_async(
        &self,
        sender_addr: String,
        recips: Vec<String>,
        body_text: String,
    ) -> Result<(), String> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || -> Result<(), String> {
            let mut conn = pool.get().map_err(|e| e.to_string())?;
            Self::save_email(&mut conn, sender_addr, recips, body_text).map_err(|e| e.to_string())
        })
        .await
        .map_err(|e| e.to_string())
        .unwrap()?;

        Ok(())
    }

    pub fn save_email(
        conn: &mut SqliteConnection,
        sender_addr: String,
        recips: Vec<String>,
        body_text: String,
    ) -> QueryResult<()> {
        let new_email = NewEmail {
            sender: sender_addr,
            subject: None,
            body: body_text,
        };

        diesel::insert_into(emails::table)
            .values(&new_email)
            .execute(conn)?;

        // Get the ID using the SQLite helper
        let last_id: i32 = diesel::select(diesel::dsl::sql::<diesel::sql_types::Integer>(
            "last_insert_rowid()",
        ))
        .get_result(conn)?;

        for r in recips {
            let new_recipient = NewRecipient {
                email_id: last_id,
                recipient: r,
            };

            diesel::insert_into(recipients::table)
                .values(&new_recipient)
                .execute(conn)?;
        }

        Ok(())
    }

    pub async fn create_user(&self, mail: String, password: String) -> Result<User, String> {
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || -> Result<User, String> {
            use crate::schema::users::dsl::*;

            let mut conn = pool.get().map_err(|e| e.to_string())?;

            let new_user = NewUser {
                email: mail.clone(),
                password_hash: password,
            };

            diesel::insert_into(users)
                .values(&new_user)
                .execute(&mut conn)
                .map_err(|e| e.to_string())?;

            let inserted_user = users
                .filter(email.eq(mail))
                .first::<User>(&mut conn)
                .map_err(|e| e.to_string())?;

            Ok(inserted_user)
        })
        .await
        .map_err(|e| e.to_string())?
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, String> {
        let pool = self.pool.clone();
        let username = email.to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| e.to_string())?;

            users::table
                .filter(users::email.eq(username))
                .first::<User>(&mut conn)
                .optional()
                .map_err(|e| e.to_string())
        })
        .await
        .map_err(|e| e.to_string())?
    }

    pub async fn verify_password(&self, email: &str, password: &str) -> Result<bool, String> {
        let user = self.get_user_by_email(email).await?;

        match user {
            Some(user) => {
                let stored_password = user.password_hash;

                if stored_password.starts_with("$argon2") {
                    Ok(false)
                } else {
                    Ok(stored_password == password)
                }
            }
            None => Ok(false),
        }
    }

    pub async fn get_mails_async(&self, user: String) -> Result<Vec<Email>, String> {
        let pool = self.pool.clone();

        let mails = tokio::task::spawn_blocking(move || -> Result<Vec<Email>, String> {
            let mut conn = pool.get().map_err(|e| e.to_string())?;
            Self::get_mails(&mut conn, user).map_err(|e| e.to_string())
        })
        .await
        .map_err(|e| e.to_string())
        .unwrap()?;

        Ok(mails)
    }

    pub fn get_mails(
        conn: &mut SqliteConnection,
        user: String,
    ) -> Result<Vec<Email>, diesel::result::Error> {
        let results = emails::table
            .inner_join(recipients::table.on(recipients::email_id.eq(emails::id)))
            .filter(recipients::recipient.eq(user))
            .select(Email::as_select())
            .load::<Email>(conn)?;

        Ok(results)
    }

    pub async fn verify_email(&self, email: String) -> Result<bool, String> {
        let pool = self.pool.clone();

        let exist = tokio::task::spawn_blocking(move || -> Result<bool, String> {
            let mut conn = pool.get().map_err(|e| e.to_string())?;

            let res = diesel::select(exists(
                recipients::table.filter(recipients::recipient.eq(email)),
            ))
            .get_result::<bool>(&mut conn)
            .map_err(|e| e.to_string())?;

            Ok(res)
        })
        .await
        .map_err(|e| e.to_string())??;

        Ok(exist)
    }

    pub async fn get_users(&self) -> Result<Vec<User>, String> {
        let pool = self.pool.clone();

        let existing_users = tokio::task::spawn_blocking(move || -> Result<Vec<User>, String> {
            let mut conn = pool.get().map_err(|e| e.to_string())?;

            users::table
                .select(User::as_select())
                .load(&mut conn)
                .map_err(|e| e.to_string())
        })
        .await
        .map_err(|e| e.to_string())??;

        Ok(existing_users)
    }
}
