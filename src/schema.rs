// @generated automatically by Diesel CLI.

diesel::table! {
    tokens (id) {
        index -> Int4,
        id -> Nullable<Text>,
        count -> Nullable<Int4>,
        bracket -> Nullable<Int4>,
        level -> Nullable<Text>,
    }
}
