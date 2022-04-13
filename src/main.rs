extern crate clap;
extern crate reqwest;
extern crate rust_decimal;
mod commands;
mod config;
mod fdf;
mod forms;
use clap::{AppSettings, Parser, Subcommand};
use commands::tax_breakdown::TaxBreakdownArgs;
use commands::{ams::AmsArgs, tax_breakdown};
use std::{fs::File, path::Path};

use crate::{commands::ams, config::Config};

#[derive(Parser, Debug)]
#[clap(about, version, author)]
#[clap(setting(AppSettings::SubcommandRequiredElseHelp))]
struct CliArgs {
    #[clap(subcommand)]
    command: Commands,
    #[clap(
        short,
        long,
        help = "Path to config file",
        default_value = ".fbihtax.json"
    )]
    config: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Ams(AmsArgs),
    TaxBreakdown(TaxBreakdownArgs),
}

fn main() {
    let args = CliArgs::parse();

    let config: Config = config::parse_config_with_default(args.config.as_str());

    if !Path::new(config.pdf.cache_location.as_str()).exists() {
        println!(
            "Cached form not found at: {}\nResorting to download from: {}",
            config.pdf.cache_location, config.pdf.download_url
        );
        let mut result = reqwest::blocking::get(config.pdf.download_url.to_string())
            .expect("Failed downloading form PDF");
        let mut file_writer =
            File::create(config.pdf.cache_location.as_str()).expect("Failed creating cache file");
        result
            .copy_to(&mut file_writer)
            .expect("Failed saving downloaded PDF");
        println!(
            "Downloaded form and cached to: {}",
            config.pdf.cache_location
        );
    }

    match &args.command {
        Commands::Ams(ams_args) => ams::handle_command(config, ams_args),
        Commands::TaxBreakdown(tax_breakdown_args) => {
            tax_breakdown::handle_command(config, tax_breakdown_args)
        }
    }
}
