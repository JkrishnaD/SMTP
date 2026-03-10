use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use crate::models::{Email, NewEmail, NewRecipient};
use crate::schema::{emails, recipients};

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

pub fn get_mails(
    conn: &mut SqliteConnection,
    user: String,
) -> Result<Vec<Email>, diesel::result::Error> {
    // Use explicit paths to avoid "cannot find table" errors
    let results = emails::table
        .inner_join(recipients::table.on(recipients::email_id.eq(emails::id)))
        .filter(recipients::recipient.eq(user))
        .select(Email::as_select()) // Requires #[derive(Selectable)] on Email
        .load::<Email>(conn)?;

    Ok(results)
}
