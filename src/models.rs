use super::schema::charades;
use bigdecimal::BigDecimal;

#[derive(Insertable)]
#[table_name="charades"]
pub struct NewCharade<'a> {
    pub category: &'a str,
    pub hint: &'a str,
    pub puzzle: &'a str,
}
#[derive(Queryable)]
pub struct Charade {
    pub id: i32,
    pub category: String,
    pub hint: String,
    pub puzzle: String,
    pub userid: BigDecimal,
    pub public: bool,
}