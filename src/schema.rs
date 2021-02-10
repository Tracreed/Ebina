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
