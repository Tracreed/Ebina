use super::schema::charades;
use super::schema::*;
use bigdecimal::BigDecimal;

#[derive(Insertable)]
#[table_name = "charades"]
pub struct NewCharade<'a> {
    pub category: &'a Categories,
    pub hint: &'a str,
    pub puzzle: &'a str,
    pub solution: &'a str,
    pub difficulty: &'a Difficulties,
    pub userid: &'a BigDecimal,
    pub public: &'a bool,
}
#[derive(Queryable)]
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

#[derive(Queryable, Debug)]
pub struct Feed {
    pub id: i32,
    pub server: i64,
    pub channel: i64,
    pub manga: String,
}

#[derive(Insertable)]
#[table_name = "feeds"]
pub struct NewFeed<'a> {
    pub server_id: &'a i64,
    pub channel_id: &'a i64,
    pub manga_id: &'a String,
}

#[derive(Queryable, Debug)]
pub struct Role {
    pub id: i32,
    pub server: i64,
    pub data: String,
}

#[derive(Insertable)]
#[table_name = "roles"]
pub struct NewRole<'a> {
    pub server_id: &'a i64,
    pub data: &'a String,
}
