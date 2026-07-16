use std::{error::Error, fs};

use tokio_rusqlite::{Connection};

pub struct Repository{
    pub connection: Connection
}

impl Repository{
    pub async fn new() -> Result<Self, Box<dyn Error>>{
        let mut path = dirs::config_dir().expect("failed get config directory");
        path.push("sarcent");
        if !path.exists(){
            fs::create_dir_all(&path).expect("failed create config directory");
        }
        path.push("app.db");
        let path_db = path.to_str().unwrap();

        let connection = Connection::open(path_db).await?;

        Ok(Self{connection})
    }
}
