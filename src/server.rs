use cmd_args::CmdArgs;

pub fn run() -> Result<(), String> {
    let args = CmdArgs::new();

    if args.print_version {
        println!("Livy Manager {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    Ok(())
}
