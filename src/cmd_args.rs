use argparse::{ArgumentParser, Store, StoreTrue};

/// Command-line arguments
#[derive(Debug)]
pub struct CmdArgs {
    pub conf_path: String,
    pub print_version: bool,
}

impl CmdArgs {
    /// Creates a new `CmdArgs`.
    pub fn new() -> CmdArgs {
        let mut conf_path = String::new();
        let mut print_version = false;

        {
            let mut p = ArgumentParser::new();
            p.set_description("Livy Manager - Web UI for Managing Apache Livy Sessions");
            p.refer(&mut conf_path)
                .add_option(&["-c", "--conf-path"],
                            Store,
                            "Configuration file path");
            p.refer(&mut print_version)
                .add_option(&["-V"],
                            StoreTrue,
                            "Print version info and exit");
            p.parse_args_or_exit()
        }

        CmdArgs {
            conf_path,
            print_version,
        }
    }
}
