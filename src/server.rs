use cmd_args::CmdArgs;
use config::Config;
use hyper::server::Http;
use livy_manager::LivyManager;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub fn run() {
    let args = CmdArgs::new();

    if args.print_version {
        println!("Livy Manager {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    let conf = Config::from(&args.conf_path);
    let user_sessions = Arc::new(Mutex::new(HashMap::new()));

    let addr = conf.http.addr.parse().unwrap();
    let server = Http::new()
        .bind(&addr, move || Ok(LivyManager::new(conf.clone(), Arc::clone(&user_sessions))))
        .unwrap();

    eprintln!("Livy Manager {}", env!("CARGO_PKG_VERSION"));
    eprintln!("Listening on {}.", addr);

    server.run().unwrap();
}

/// User session
#[derive(Clone, Debug)]
pub struct UserSession {
    pub uid: String,
    pub is_admin: bool,
}

pub type UserSessions = HashMap<String, UserSession>;
