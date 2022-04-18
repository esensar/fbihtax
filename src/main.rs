extern crate clap;
extern crate reqwest;
extern crate rust_decimal;
mod commands;
mod config;
mod fdf;
mod format;
mod forms;
use clap::{AppSettings, Parser, Subcommand};
use commands::ams::{self, AmsArgs};
use commands::gpd::{self, GpdArgs};
use commands::taxbreakdown::{self, TaxBreakdownArgs};
use config::Config;

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
    Gpd(GpdArgs),
    TaxBreakdown(TaxBreakdownArgs),
}

fn main() {
    let args = CliArgs::parse();

    let config: Config = config::parse_config_with_default(args.config.as_str());

    match &args.command {
        Commands::Ams(ams_args) => ams::handle_command(config, ams_args),
        Commands::Gpd(gpd_args) => gpd::handle_command(config, gpd_args),
        Commands::TaxBreakdown(tax_breakdown_args) => {
            taxbreakdown::handle_command(config, tax_breakdown_args)
        }
    }
}
