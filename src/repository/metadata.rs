
use std::{error::Error, fs};

use tokio_rusqlite::{Connection};

pub struct MetadataItem{
    pub name: String,
    pub path: String,
    pub size_bytes: u32,
}


pub struct MetadataRepository{
    pub conn: Connection
}

impl MetadataRepository {
    pub async fn new()-> Result<Self, Box<dyn std::error::Error>>{
        let mut path = dirs::config_dir().expect("failed get config directory");
        path.push("sarcent");
        if !path.exists(){
            fs::create_dir_all(&path).expect("failed create config directory");
        }
        path.push("app.db");
        let path_db = path.to_str().unwrap();

        let conn = Connection::open(path_db).await?;

        conn.call(|conn|{
            conn.execute("CREATE TABLE IF NOT EXISTS metadata (
                name TEXT NOT NULL,
                path TEXT NOT NULL,
                size_bytes BIGINT,
                discovered_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            )", [])
        }).await?;

        Ok(Self{conn})
    }

    pub async fn add_metadata(&self, metadata:MetadataItem ) -> Result<(), Box<dyn Error>>{
        let query: &str = "INSERT INTO metadata(name, path, size_bytes) VALUES($1, $2, $3)";
        self.conn.call(move |conn|{
            conn.execute(query, [metadata.name, metadata.path, metadata.size_bytes.to_string()])
        }).await?;
        Ok(())
    }

    // fn get_files(){
    //
    // }
    
    
}
