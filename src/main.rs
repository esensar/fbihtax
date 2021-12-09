extern crate clap;
extern crate reqwest;
extern crate rust_decimal;
mod amsform;
mod config;
use clap::Parser;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::{fs::File, path::Path};

use amsform::FormField;

use crate::config::{ClientConfig, Config, UserConfig};

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct CliArgs {
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

    let income_dec: Decimal = args.income.round_dp(2);
    let deduction_factor: Decimal = dec!(1) - (args.deduction_percentage.round_dp(2) * dec!(0.01));
    form.add_income(income_dec * deduction_factor, dec!(0));

    let output_path = Path::new(config.output_location.as_str());
    let output_file_path = output_path.join("amsform.pdf");
    let output_file_path_str = output_file_path
        .to_str()
        .expect("Output location seems to be invalid!");
    form.save(output_file_path_str);
    println!("Saved output form to: {}", output_file_path_str);
}
