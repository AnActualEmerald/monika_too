use crate::models::*;
use crate::schema::configs::dsl::*;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

pub struct Database {
    connection: SqliteConnection,
}

impl Database {
    pub fn new(url: &str) -> Database {
        let connection = SqliteConnection::establish(url).expect("Unable to connect to database");
        Database {
            connection: connection,
        }
    }

    pub fn add_server(&self, id: u64) {
        diesel::insert_into(configs)
            .values(NewConfig {
                server_id: &(id as i32),
                prefix: "!",
            })
            .execute(&self.connection)
            .unwrap();
    }

    pub fn set_prefix(&self, id: u64, prf: &str) {
        diesel::replace_into(configs)
            .values((server_id.eq(id as i32), prefix.eq(prf.to_string())))
            .execute(&self.connection)
            .unwrap();
    }

    pub fn get_prefix<'a>(&self, id: u64) -> Option<String> {
        let server = configs
            .filter(server_id.eq(id as i32))
            .limit(1)
            .load::<Config>(&self.connection)
            .unwrap();

        if server.len() == 0 {
            return None;
        }

        Some(server[0].prefix.clone())
    }

    pub fn get_server_config(&self, id: u64) -> Option<Vec<Config>> {
        let server = configs
            .filter(server_id.eq(id as i32))
            .limit(1)
            .load::<Config>(&self.connection)
            .unwrap();

        if server.len() == 0 {
            return None;
        }

        Some(server)
    }
}
