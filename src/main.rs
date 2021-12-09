extern crate clap;
extern crate rust_decimal;
mod amsform;
mod config;
use clap::Parser;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::path::Path;

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
    let mut form = amsform::load_ams_form(config.pdf.cache_location);

    form.fill_main_field(FormField::UserName, "Ensar".to_string());
    let income_dec: Decimal = args.income.round_dp(2);
    form.add_income(income_dec, dec!(0));

    form.save(
        Path::new(config.output_location.as_str())
            .join("amsform.pdf")
            .to_str()
            .expect("Output location seems to be invalid!"),
    );
}
