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

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct CliArgs {
    #[clap(short, long, default_value = ".fbihtax.json")]
    config: String,
    #[clap(short, long)]
    income: Decimal,
}

fn main() {
    let args = CliArgs::parse();

    let config = config::parse_config(args.config.as_str());

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

    form.fill_main_field(FormField::UserName, "Ensar".to_string());
    let income_dec: Decimal = args.income.round_dp(2);
    form.add_income(income_dec, dec!(0));

    let output_path = Path::new(config.output_location.as_str());
    let output_file_path = output_path.join("amsform.pdf");
    let output_file_path_str = output_file_path
        .to_str()
        .expect("Output location seems to be invalid!");
    form.save(output_file_path_str);
    println!("Saved output form to: {}", output_file_path_str);
}
