// @generated automatically by Diesel CLI.

diesel::table! {
    emails (id) {
        id -> Nullable<Integer>,
        sender -> Text,
        subject -> Nullable<Text>,
        body -> Text,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    recipients (id) {
        id -> Nullable<Integer>,
        email_id -> Integer,
        recipient -> Text,
    }
}

diesel::joinable!(recipients -> emails (email_id));

diesel::allow_tables_to_appear_in_same_query!(emails, recipients,);
