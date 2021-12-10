extern crate clap;
extern crate reqwest;
extern crate rust_decimal;
mod amsform;
mod config;
use clap::{AppSettings, Parser, Subcommand};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::json;
use std::{fs::File, path::Path};

use amsform::FormField;

use crate::config::{ClientConfig, Config, UserConfig};

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
    #[clap(
        short,
        long,
        help = "Decimal income value in BAM (will be rounded to 2 decimals)"
    )]
    income: Decimal,
    #[clap(
        short,
        long,
        help = "Tax deduction percentage (20 default, 30 for income from authored work)",
        default_value_t = dec!(20)
    )]
    deduction_percentage: Decimal,
    #[clap(long, help = "Path to config file with user specific settings")]
    user_config: Option<String>,
    #[clap(long, help = "Path to config file with client specific settings")]
    client_config: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Ams,
    TaxBreakdown,
}

fn main() {
    let args = CliArgs::parse();

    let config: Config = config::parse_config_with_default(args.config.as_str());

    if !Path::new(config.pdf.cache_location.as_str()).exists() {
        println!(
            "Cached form not found at: {}\nResorting to download from: {}",
            config.pdf.cache_location, config.pdf.download_url
        );
        let mut result =
            reqwest::blocking::get(config.pdf.download_url).expect("Failed downloading form PDF");
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

    let income_dec: Decimal = args.income.round_dp(2);
    let deduction_factor: Decimal = dec!(1) - (args.deduction_percentage.round_dp(2) * dec!(0.01));
    let income_after = income_dec * deduction_factor;

    match &args.command {
        Commands::Ams => {
            let mut form = amsform::load_ams_form(config.pdf.cache_location);

            let user_config = match args.user_config {
                Some(path) => config::parse_config::<UserConfig>(path.as_str()),
                None => config.user,
            }.expect("Missing user configuration. Either fill it in default config file or pass --user-config parameter.");
            form.fill_main_field(FormField::UserName, user_config.name);
            form.fill_main_field(FormField::UserAddress, user_config.address);
            form.fill_main_field(FormField::UserJmbg, user_config.jmbg);

            let client_config = match args.client_config {
                Some(path) => config::parse_config::<ClientConfig>(path.as_str()),
                None => config.client,
            }.expect("Missing client configuration. Either fill it in default config file or pass --client-config parameter.");
            form.fill_main_field(FormField::CompanyName, client_config.name);
            form.fill_main_field(FormField::CompanyAddress, client_config.address);
            form.fill_main_field(FormField::CompanyCountry, client_config.country);

            form.add_income(income_after, dec!(0));

            let output_path = Path::new(config.output_location.as_str());
            let output_file_path = output_path.join("amsform.pdf");
            let output_file_path_str = output_file_path
                .to_str()
                .expect("Output location seems to be invalid!");
            form.save(output_file_path_str);
            println!("Saved output form to: {}", output_file_path_str);
        }
        &Commands::TaxBreakdown => {
            let health_insurance = income_after * dec!(0.04);
            let tax_base = income_after - health_insurance;
            let tax_amount: Decimal = tax_base * dec!(0.10);
            let health_insurance_federation = health_insurance * dec!(0.1020);
            let health_insurance_canton = health_insurance - health_insurance_federation; // or *0.8980, but this is more accurate

            let output_path = Path::new(config.output_location.as_str());
            let output_file_path = output_path.join("taxbreakdown.json");
            let output_file_path_str = output_file_path
                .to_str()
                .expect("Output location seems to be invalid!");
            let breakdown_writer =
                File::create(output_file_path_str).expect("Failed creating tax breakdown file");
            let json = json!({
                "income_tax": tax_amount.round_dp(2),
                "health_insurance": {
                    "federation": health_insurance_federation.round_dp(2),
                    "canton": health_insurance_canton.round_dp(2),
                    "total": health_insurance.round_dp(2)
                },
                "total": (health_insurance + tax_amount).round_dp(2)
            });
            serde_json::to_writer_pretty(breakdown_writer, &json)
                .expect("Failed saving downloaded PDF");
            println!("Saved tax breakdown to: {}", output_file_path_str);
        }
    }
}
