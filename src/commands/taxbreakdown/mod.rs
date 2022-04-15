extern crate clap;
extern crate rust_decimal;

mod printer;

use std::path::Path;

use clap::Parser;
use printer::{JsonPrinter, OutputFormat, StdoutPrinter, TaxBreakdownPrinter};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::config::Config;

use self::printer::TaxBreakdownData;

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
    #[clap(long, help = "Output format (JSON, stdout)", default_value_t = OutputFormat::Json)]
    output_format: OutputFormat,
}

pub fn handle_command(config: Config, args: &TaxBreakdownArgs) {
    let json_printer = JsonPrinter {};
    let stdout_printer = StdoutPrinter {};

    let printer: &dyn TaxBreakdownPrinter = match args.output_format {
        OutputFormat::Json => &json_printer,
        OutputFormat::Stdout => &stdout_printer,
    };

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
    let data = TaxBreakdownData {
        income_tax: tax_amount,
        health_insurance_federation,
        health_insurance_canton,
    };
    printer.write_to_file(&data, output_file_path_str);
}
