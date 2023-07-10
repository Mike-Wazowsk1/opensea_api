// @generated automatically by Diesel CLI.

diesel::table! {
    tokens (id) {
        id -> Text,
        count -> Nullable<Int4>,
        bracket -> Nullable<Int4>,
        level -> Nullable<Text>,
    }
}
