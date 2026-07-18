use std::{error::Error};

use tabled::Tabled;
use tokio_rusqlite::{Connection};

use crate::repository::base::Repository;

pub struct MetadataItem{
    pub name: String,
    pub path: String,
    pub size_bytes: u32,
    pub server_address: String,
}

#[derive(Debug, Tabled)]
pub struct MetadataRowItem {
    pub name: String,
    pub path: String,
    pub size_bytes: u32,
    pub discovered_at: String,
    pub server_address: String,
    pub created_at: String,
    pub updated_at: String,
}


pub struct MetadataRepository{
    pub conn: Connection
}

impl MetadataRepository {
    pub async fn new()-> Result<Self, Box<dyn std::error::Error>>{
        let repository = Repository::new().await?; 

        // conn.call(|conn|{
        //     conn.execute("CREATE TABLE IF NOT EXISTS metadata (
        //         name TEXT NOT NULL,
        //         path TEXT NOT NULL,
        //         size_bytes BIGINT,
        //         discovered_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        //         server_address TEXT NOT NULL,
        //         created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        //         updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        //     );
        //     CREATE INDEX IF NOT EXISTS idx_metadata_name on metadata(name);
        //     ", [])
        // }).await?;
        //
        // conn.call(|conn|{
        //     conn.execute("", [])
        // }).await?;
        //
        Ok(Self{conn: repository.connection})
    }

    pub async fn add_metadata(&self, metadata:MetadataItem ) -> Result<(), Box<dyn Error>>{
        let query: &str = "INSERT INTO metadata(name, path, size_bytes, server_address) VALUES(?1, ?2, ?3, ?4)";
        self.conn.call(move |conn|{
            conn.execute(query, [metadata.name, metadata.path, metadata.size_bytes.to_string(), metadata.server_address])
        }).await?;
        Ok(())
    }

    pub async fn search_by_name(&self, name: String) -> Result<Vec<MetadataRowItem>, tokio_rusqlite::Error>{
        let search_param = format!("%{}%", name);
        let query: &str = "SELECT * FROM metadata WHERE name LIKE ?1";

        let files = self.conn.call(|conn|{
            let mut stmt = conn.prepare(query)?;
            let rows = stmt.query_map([search_param], |row| {
               let item = MetadataRowItem{
                    name: row.get(0)?,
                    path: row.get(1)?,
                    size_bytes: row.get(2)?,
                    discovered_at: row.get(3)?,
                    server_address: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                }; 
                Ok(item)
            })?;

            rows.collect::<Result<Vec<MetadataRowItem>, _>>()
        }).await?;

        Ok(files)
    }

    // fn get_files(){
    //
    // }
    
    
}
