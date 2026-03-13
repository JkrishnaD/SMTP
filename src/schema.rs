// @generated automatically by Diesel CLI.

diesel::table! {
    emails (id) {
        id -> Integer,
        sender -> Text,
        subject -> Nullable<Text>,
        body -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    recipients (id) {
        id -> Integer,
        email_id -> Integer,
        recipient -> Text,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        email -> Text,
        password_hash -> Text,
        created_at -> Timestamp,
    }
}

diesel::joinable!(recipients -> emails (email_id));

diesel::allow_tables_to_appear_in_same_query!(emails, recipients, users,);
