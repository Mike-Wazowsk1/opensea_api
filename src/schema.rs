// @generated automatically by Diesel CLI.

diesel::table! {
    info (hash) {
        hash -> Text,
        wbgl -> Nullable<Int4>,
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
diesel::table! {
    info_lotto (last_payment) {
        last_payment -> Text,
        wining_block -> Nullable<Int4>,
        round -> Nullable<Int4>,
        wbgl -> Nullable<Int4>

    }
}

diesel::allow_tables_to_appear_in_same_query!(info, tokens,info_lotto);
