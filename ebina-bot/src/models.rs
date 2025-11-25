use bigdecimal::BigDecimal;
use sqlx::FromRow;

#[derive(FromRow)]
pub struct Charade {
    pub id: i32,
    pub category: Categories,
    pub hint: String,
    pub puzzle: String,
    pub solution: String,
    pub difficulty: Difficulties,
    pub userid: BigDecimal,
    pub public: bool,
}

#[derive(FromRow, Debug)]
pub struct Feed {
    pub id: i32,
    pub server_id: i64,
    pub channel_id: i64,
    pub manga_id: String,
}

#[derive(FromRow, Debug)]
pub struct Role {
    pub id: i32,
    pub server_id: i64,
    pub data: String,
}

#[derive(FromRow, Debug)]
pub struct ServerSettings {
    pub id: i32,
    pub server_id: i64,
    pub prefix: String,
}

#[derive(Debug, PartialEq, Clone, sqlx::Type)]
#[sqlx(type_name = "Categories")]
pub enum Categories {
    Anime,
    Manga,
    Game,
    TV,
    Movie,
}

#[derive(Debug, PartialEq, Clone, sqlx::Type)]
#[sqlx(type_name = "Difficulties")]
pub enum Difficulties {
    Easy,
    Medium,
    Hard,
}
