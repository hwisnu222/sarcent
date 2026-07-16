
use std::{error::Error, fs};

use tokio_rusqlite::{Connection, rusqlite};

pub struct VnodeRepository{
    pub conn: Connection
}

impl VnodeRepository {
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
            conn.execute("CREATE TABLE IF NOT EXISTS vnodes (
                server_address TEXT NOT NULL,
                discovered_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )", [])
        }).await?;

        Ok(Self{conn})
    }
    
    pub async fn add_node(&self, server: String) -> Result<(), Box<dyn std::error::Error>>{
        let server_address = server.clone();

        self.conn.call(move |conn|{
            conn.execute("INSERT OR REPLACE INTO vnodes (server_address) VALUES(?1)", rusqlite::params![server_address])
        }).await?;
        Ok(())
    }

    // remove node by server_address
    pub async fn remove_node(&self, server_address: String) -> Result<usize, Box<dyn Error>>{
        let node_removed = self.conn.call(move |conn|{
            conn.execute("DELETE FROM vnodes WHERE server_address = ?1", [server_address])
        }).await?;

        Ok(node_removed)
    }

    pub async fn get_servers(&self) -> Result<Vec<String>, Box<dyn Error>>{
        let servers = self.conn.call(|conn| -> Result<Vec<String>, tokio_rusqlite::Error>{
            let mut stmt = conn.prepare("SELECT DISTINCT server_address FROM vnodes ORDER BY created_at ASC")?;

            let mut server_list = Vec::new();
            let mut rows = stmt.query([])?;

            while let Some(row) = rows.next()?{
                let server = row.get(0)?;
                server_list.push(server);
            };

            Ok(server_list)
        }).await?;
        Ok(servers)
    }

    
}
