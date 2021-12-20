extern crate clap;
extern crate rust_decimal;

use std::path::Path;

use clap::Parser;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::{
    amsform::{self, FormField},
    config::{self, ClientConfig, Config, UserConfig},
};

#[derive(Parser, Debug)]
pub struct AmsArgs {
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

pub fn handle_command(config: Config, args: &AmsArgs) {
    let mut form = amsform::load_ams_form(config.pdf.cache_location);

    let income_dec: Decimal = args.income.round_dp(2);
    let deduction_factor: Decimal = dec!(1) - (args.deduction_percentage.round_dp(2) * dec!(0.01));
    let income_after = income_dec * deduction_factor;

    let user_config = match &args.user_config {
        Some(path) => config::parse_config::<UserConfig>(path.as_str()),
        None => config.user,
    }.expect("Missing user configuration. Either fill it in default config file or pass --user-config parameter.");
    form.fill_main_field(FormField::UserName, user_config.name);
    form.fill_main_field(FormField::UserAddress, user_config.address);
    form.fill_main_field(FormField::UserJmbg, user_config.jmbg);

    let client_config = match &args.client_config {
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
