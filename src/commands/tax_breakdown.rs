extern crate clap;
extern crate rust_decimal;

use std::{fs::File, path::Path};

use clap::Parser;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::json;

use crate::config::Config;

#[derive(Parser, Debug)]
pub struct TaxBreakdownArgs {
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
    #[clap(
        short,
        long,
        help = "Path to save output JSON to",
        default_value = "taxbreakdown.json"
    )]
    output: String,
}

pub fn handle_command(config: Config, args: &TaxBreakdownArgs) {
    let income_dec: Decimal = args.income.round_dp(2);
    let deduction_factor: Decimal = dec!(1) - (args.deduction_percentage.round_dp(2) * dec!(0.01));
    let income_after = income_dec * deduction_factor;

    let health_insurance = income_after * dec!(0.04);
    let tax_base = income_after - health_insurance;
    let tax_amount: Decimal = tax_base * dec!(0.10);
    let health_insurance_federation = health_insurance * dec!(0.1020);
    let health_insurance_canton = health_insurance - health_insurance_federation; // or *0.8980, but this is more accurate

    let output_path = Path::new(config.output_location.as_str());
    let output_file_path = output_path.join(&args.output);
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
    serde_json::to_writer_pretty(breakdown_writer, &json).expect("Failed saving downloaded PDF");
    println!("Saved tax breakdown to: {}", output_file_path_str);
}
