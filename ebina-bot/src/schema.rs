table! {
    use crate::models::{DifficultiesMapping, CategoriesMapping};
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
    discord_settings (id) {
        id -> Int4,
        server_id -> Int8,
        prefix -> Varchar,
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

allow_tables_to_appear_in_same_query!(charades, discord_settings, feeds, roles,);
