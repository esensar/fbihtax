extern crate clap;
extern crate rust_decimal;

mod insert;
mod load;

use crate::config::Config;
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
    Load(LoadArgs),
    Insert(InsertArgs),
}

pub fn handle_command(config: Config, args: &DbArgs) {
    match &args.command {
        DbCommands::Load(load_args) => load::handle_command(config, load_args),
        DbCommands::Insert(insert_args) => insert::handle_command(config, insert_args),
    }
}
