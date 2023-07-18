// @generated automatically by Diesel CLI.

diesel::table! {
    info (hash) {
        hash -> Text,
        wbgl -> Nullable<Float8>,
    }
}

diesel::table! {
    tokens (index) {
        index -> Int4,
        id -> Nullable<Text>,
        count -> Nullable<Int4>,
        bracket -> Nullable<Int4>,
        level -> Nullable<Text>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    info,
    tokens,
);
