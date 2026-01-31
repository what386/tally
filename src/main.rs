mod application;
mod models;
mod services;
mod utils;

use console::style;

use clap::Parser;

use application::cli::arguments::Cli;

fn main() {
    let cli = Cli::parse();

    if let Err(err) = cli.run() {
        #[cfg(debug_assertions)]
        {
            eprintln!("{:?}", style(err).red());
        }

        #[cfg(not(debug_assertions))]
        {
            eprintln!(
                "{}",
                style(
                    err.chain()
                        .map(|e| e.to_string())
                        .collect::<Vec<_>>()
                        .join("\n")
                )
                .red()
            );
        }

        std::process::exit(1);
    }
}
