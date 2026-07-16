use std::{error::Error};
use tokio_rusqlite::{Connection};
use tracing::{debug};
use walkdir::WalkDir;

use crate::repository::base::Repository;

pub struct MigrateDatabase{
   pub conn: Connection
}

impl MigrateDatabase{
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let repository = Repository::new().await?;
        let conn = repository.connection;
        Ok(Self{conn})
    }

    // migrate all file sqlite from file sql instead of run query
    // inside of each repository
    pub async fn migrate(&self) -> Result<(), Box<dyn Error>>{
        for entry in WalkDir::new("./src/migrations/"){
            let entry_file = entry?;
            let path = entry_file.path();

            if path.is_file(){
                debug!("{} is migrated", path.file_name().unwrap().to_string_lossy());
                let sql = include_str!("../migrations/metadata.sql");

                self.conn.call(|conn|{
                    conn.execute_batch(sql)
                }).await?;
            }

        }
        Ok(())
    }
}
