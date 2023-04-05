table! {
    tasks (id) {
        id -> Integer,
        name -> Text,
        started_at -> BigInt,
        finished_at -> Nullable<BigInt>,
    }
}
