use std::{fs::File};

use daemonize::Daemonize;

pub struct Daemon{
    pub name: String
}

impl Daemon{
    pub fn new(name: &str)-> Self{
        Self{name: name.to_string()}
    }

    pub fn run_background(&self){
        let stdout = File::create(format!("/tmp/{}.out", self.name)).unwrap();
        let stderr = File::create(format!("/tmp/{}.err", self.name)).unwrap();

        let daemonize = Daemonize::new()
            .pid_file(format!("/tmp/{}.pid", self.name))
            .chown_pid_file(true)
            .working_directory("/tmp")
            .stdout(stdout)
            .stderr(stderr);
            
        if let Err(e) = daemonize.start() {
            eprintln!("failed run process to background. Error: {}", e);
            std::process::exit(1);
        }
    }


}
