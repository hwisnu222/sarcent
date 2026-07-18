use std::{error::Error};
use tokio_rusqlite::{Connection};
use tracing::{debug};

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
        // TODO: change this vector read from folder
        // with the include_dir crate
        let migrations = vec![
            include_str!("../migrations/metadata.sql"),
            include_str!("../migrations/vnode.sql")
        ];

        debug!("migrate...");
        for migration in migrations{
            self.conn.call(|conn|{
                conn.execute_batch(migration)
            }).await?;
        };

        Ok(())
    }
}
