use diesel::prelude::*;
use crate::schema::tokens;


#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::tokens)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Token {
    pub index:i32,
    pub id: Option<String>,
    pub count: Option<i32>,
    pub bracket: Option<i32>,
    pub level:  Option<String>
}


#[derive(Insertable)]
#[diesel(table_name = tokens)]
pub struct NewToken<'a> {
    pub index: &'a i32,
    pub id: &'a str,
    pub count: &'a i32,
    pub bracket: &'a i32,
    pub level: &'a str,
}