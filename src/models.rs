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
