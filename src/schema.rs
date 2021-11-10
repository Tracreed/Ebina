use diesel_derive_enum::DbEnum;
#[derive(Debug, PartialEq, DbEnum, Clone)]
pub enum Categories {
    Anime, // All variants must be fieldless
    Manga,
    Game,
    TV,
    Movie,
}
#[derive(Debug, PartialEq, DbEnum, Clone)]
pub enum Difficulties {
    Easy, // All variants must be fieldless
    Medium,
    Hard,
}

table! {
    use super::{DifficultiesMapping, CategoriesMapping};
    use diesel::sql_types::*;
    charades (id) {
        id -> Int4,
        category -> CategoriesMapping,
        hint -> Text,
        puzzle -> Text,
        solution -> Text,
        difficulty -> DifficultiesMapping,
        userid -> Numeric,
        public -> Bool,
    }
}

table! {
    feeds (id) {
        id -> Int4,
        server_id -> Int8,
        channel_id -> Int8,
        manga_id -> Text,
    }
}

table! {
    roles (id) {
        id -> Int4,
        server_id -> Int8,
        #[sql_name = "roles"]
        data -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    charades,
    feeds,
    roles,
);
