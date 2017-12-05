use cmd_args::CmdArgs;
use config::Config;
use hyper::server::Http;
use livy_manager::LivyManager;

pub fn run() {
    let args = CmdArgs::new();

    if args.print_version {
        println!("Livy Manager {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    let conf = Config::from(&args.conf_path);

    let addr = conf.http.addr.parse().unwrap();
    let server = Http::new()
        .bind(&addr, move || Ok(LivyManager::new(conf.clone())))
        .unwrap();

    eprintln!("Livy Manager {}", env!("CARGO_PKG_VERSION"));
    eprintln!("Listening on {}.", addr);

    server.run().unwrap();
}
