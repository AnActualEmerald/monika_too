use super::schema::configs;

#[derive(Queryable)]
pub struct Config {
    pub server_id: i32,
    pub prefix: String,
}

#[derive(Insertable)]
#[table_name = "configs"]
pub struct NewConfig<'a> {
    pub server_id: &'a i32,
    pub prefix: &'a str,
}
