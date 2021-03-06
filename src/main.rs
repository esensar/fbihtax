extern crate clap;
extern crate reqwest;
extern crate rust_decimal;
mod commands;
mod config;
mod db;
mod error;
mod fdf;
mod format;
mod forms;
mod taxcalculator;
use clap::{AppSettings, Parser, Subcommand};
use commands::ams::{self, AmsArgs};
use commands::db::DbArgs;
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
    #[clap(about = "Manage AMS form (income tax payment form)")]
    Ams(AmsArgs),
    #[clap(about = "Manage GPD form (yearly tax report)")]
    Gpd(GpdArgs),
    #[clap(about = "Manage fbihtax database")]
    Db(DbArgs),
    #[clap(about = "Tax breakdown to assist with income tax payment")]
    TaxBreakdown(TaxBreakdownArgs),
}

fn main() -> error::Result<()> {
    let args = CliArgs::parse();

    let config: Config = config::parse_config_with_default(args.config.as_str());

    match &args.command {
        Commands::Ams(ams_args) => ams::handle_command(config, ams_args),
        Commands::Gpd(gpd_args) => gpd::handle_command(config, gpd_args),
        Commands::Db(db_args) => commands::db::handle_command(config, db_args),
        Commands::TaxBreakdown(tax_breakdown_args) => {
            taxbreakdown::handle_command(config, tax_breakdown_args)
        }
    }
}
