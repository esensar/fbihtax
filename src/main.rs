extern crate clap;
mod amsform;
mod config;
use clap::Parser;
use std::path::Path;

use amsform::FormField;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct CliArgs {
    #[clap(short, long, default_value = ".fbihtax.json")]
    config: String,
    #[clap(short, long)]
    income: f64,
}

fn main() {
    let args = CliArgs::parse();

    let config = config::parse_config(args.config.as_str());
    let mut form = amsform::load_ams_form(config.pdf.cache_location);

    form.fill_main_field(FormField::UserName, "Ensar".to_string());
    let income_rounded = (args.income * 100.0).round() as i64;
    form.add_income(income_rounded, 0);

    form.save(
        Path::new(config.output_location.as_str())
            .join("amsform.pdf")
            .to_str()
            .expect("Output location seems to be invalid!"),
    );
}
