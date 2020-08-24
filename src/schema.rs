table! {
    posts (id) {
        id -> Integer,
        title -> Nullable<Text>,
        body -> Nullable<Text>,
        created_at -> Timestamp,
    }
}
