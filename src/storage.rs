use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use crate::{
    models::{Email, NewEmail, NewRecipient},
    schema::{emails, recipients},
};

// Saves a new email to the database, including its sender and recipients.
pub fn save_email(conn: &mut SqliteConnection, sender: String, recips: Vec<String>, body: String) {
    let new_email = NewEmail {
        sender,
        subject: None,
        body,
    };

    let email_id = diesel::insert_into(emails::table)
        .values(&new_email)
        .returning(emails::id)
        .get_result(conn)
        .expect("Failed to insert email");

    for r in recips {
        let new_recipient = NewRecipient {
            email_id,
            recipient: r,
        };

        diesel::insert_into(recipients::table)
            .values(&new_recipient)
            .execute(conn)
            .expect("Failed to insert recipient");
    }
}

pub fn get_mails(
    conn: &mut SqliteConnection,
    user: String,
) -> Result<Vec<Email>, diesel::result::Error> {
    let results = emails::table
        .inner_join(recipients::table)
        .filter(recipients::recipient.eq(user))
        .select(emails::all_columns)
        .load(conn)?;

    Ok(results)
}
