use diesel::Insertable;
use diesel::Queryable;
use diesel::Selectable;
// use crate::schema::info;

// use crate::schema::tokens;




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
#[diesel(table_name = crate::schema::tokens)]
pub struct NewToken<'a> {
    pub index: &'a i32,
    pub id: &'a str,
    pub count: &'a i32,
    pub bracket: &'a i32,
    pub level: &'a str,
}
#[derive(Debug,Queryable, Selectable,Insertable)]
#[diesel(table_name = crate::schema::info)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InfoPoint {
    pub hash: String,
    pub wbgl:  Option<i32>

}

#[derive(Debug,Queryable, Selectable,Insertable)]
#[diesel(table_name = crate::schema::info_lotto)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InfoLottoPoint {
    pub last_payment : String,
    pub wining_block : Option<i32>,
    pub round : Option<i32>,
    pub wbgl : Option<i32>
}