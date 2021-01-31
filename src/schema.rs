table! {
    charades (id) {
        id -> Int4,
        category -> Varchar,
        hint -> Text,
        puzzle -> Text,
        userid -> Numeric,
        public -> Bool,
    }
}
