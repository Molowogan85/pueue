use anyhow::Result;
use clap::{Clap, IntoApp};
use clap_generate::generate_to;
use clap_generate::generators::*;
use simplelog::{Config, LevelFilter, SimpleLogger};

use pueue::settings::Settings;

pub mod cli;
pub mod client;
pub mod commands;
pub mod output;
pub mod output_helper;
pub mod ui;

use crate::cli::{CliArguments, Shell, SubCommand};
use crate::client::Client;

#[async_std::main]
async fn main() -> Result<()> {
    // Parse commandline options.
    let opt = CliArguments::parse();

    if let SubCommand::Completions {
        shell,
        output_directory,
    } = &opt.cmd
    {
        let mut app = CliArguments::into_app();
        app.set_bin_name("pueue");
        match shell {
            Shell::Bash => generate_to::<Bash, _, _>(&mut app, "pueue", output_directory),
            Shell::Elvish => generate_to::<Elvish, _, _>(&mut app, "pueue", output_directory),
            Shell::Fish => generate_to::<Fish, _, _>(&mut app, "pueue", output_directory),
            Shell::PowerShell => {
                generate_to::<PowerShell, _, _>(&mut app, "pueue", output_directory)
            }
            Shell::Zsh => generate_to::<Zsh, _, _>(&mut app, "pueue", output_directory),
        };
        return Ok(());
    }

    // Set the verbosity level of the logger.
    let level = match opt.verbose {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        _ => LevelFilter::Debug,
    };
    SimpleLogger::init(level, Config::default()).unwrap();

    // Try to read settings from the configuration file.
    let settings = Settings::new(true, &opt.config)?;

    // This is the entry point for the interactive UI
    // The logic for the UI is completely cut off from the rest of the client.
    if let SubCommand::Ui = &opt.cmd {
        let socket = Client::connect(&settings, &opt).await?;
        ui::run(settings, opt, socket).await?;

        return Ok(());
    }

    // Create client to talk with the daemon and connect.
    let mut client = Client::new(settings, opt).await?;

    client.start().await?;

    Ok(())
}
