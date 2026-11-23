//! `nvdf` command-line tool. Subcommands (validate, fmt, resolve, convert,
//! schema, inspect) will be added as the format is implemented.

use std::process::ExitCode;

fn main() -> ExitCode {
    let arg = std::env::args().nth(1);
    match arg.as_deref() {
        Some("--version" | "-V") => {
            println!(
                "nvdf {} (VDF format {})",
                env!("CARGO_PKG_VERSION"),
                nvdf::FORMAT_VERSION
            );
            ExitCode::SUCCESS
        }
        None | Some("--help" | "-h") => {
            println!("nvdf — VDF (Vehicle Description Format) tool\n\nUsage: nvdf --version");
            ExitCode::SUCCESS
        }
        Some(other) => {
            eprintln!("nvdf: unknown argument '{other}' (try --help)");
            ExitCode::from(2)
        }
    }
}
