extern crate clap;
extern crate rust_decimal;

mod insert;
mod load;

use crate::{config::Config, error::FbihtaxResult};
use clap::{AppSettings, Parser, Subcommand};

use self::{insert::InsertArgs, load::LoadArgs};

#[derive(Parser, Debug)]
#[clap(setting(AppSettings::SubcommandRequiredElseHelp))]
pub struct DbArgs {
    #[clap(subcommand)]
    command: DbCommands,
}

#[derive(Subcommand, Debug)]
enum DbCommands {
    #[clap(about = "Load fbihtax generated income tax PDF form (AMS form) into the database")]
    Load(LoadArgs),
    #[clap(about = "Manually insert paid income tax data into the database")]
    Insert(InsertArgs),
}

pub fn handle_command(config: Config, args: &DbArgs) -> FbihtaxResult<()> {
    match &args.command {
        DbCommands::Load(load_args) => load::handle_command(config, load_args),
        DbCommands::Insert(insert_args) => insert::handle_command(config, insert_args),
    }
}
