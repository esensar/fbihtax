extern crate clap;
extern crate rust_decimal;

mod data;

use std::{collections::HashMap, fs::File, io::Read, path::Path};

use clap::Parser;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::json;

use crate::{
    config::Config,
    format::printer::{JsonPrinter, Printer, StdoutPrinter},
    format::{utils::fill_template, OutputFormat},
};

use self::data::TaxBreakdownData;

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
    #[clap(long, help = "Output template")]
    output_template: Option<String>,
    #[clap(long, help = "Output template file path")]
    output_template_file: Option<String>,
}

fn default_json_formatter(data: HashMap<String, String>) -> serde_json::Value {
    json!({
        "income_tax": data.get("income_tax"),
        "health_insurance": {
            "federation": data.get("health_insurance_federation"),
            "canton": data.get("health_insurance_canton"),
            "total": data.get("health_insurance_total")
        },
        "total": data.get("total")
    })
}

pub fn handle_command(config: Config, args: &TaxBreakdownArgs) {
    let mut json_formatter: Box<dyn Fn(HashMap<String, String>) -> serde_json::Value> =
        Box::new(default_json_formatter);

    let mut stdout_template = concat!(
        "Income Tax breakdown:\n",
        "\n",
        "Income tax: {income_tax}\n",
        "\n",
        "Health insurance:\n",
        "  Federation: {health_insurance_federation}\n",
        "  Canton: {health_insurance_canton}\n",
        "  Total: {health_insurance_total}\n",
        "\n",
        "Total: {total}\n"
    )
    .to_string();

    match args.output_template.clone() {
        Some(template) => {
            stdout_template = template.clone();
            json_formatter = Box::new(move |data: HashMap<String, String>| -> serde_json::Value {
                let result = fill_template(template.clone(), data);
                serde_json::from_str(result.as_str()).unwrap()
            });
        }
        None => {}
    };
    match args.output_template_file.clone() {
        Some(template_file_path) => {
            let mut template_file =
                File::open(template_file_path.clone()).expect("Can't open the template file");
            let mut template = String::new();
            template_file
                .read_to_string(&mut template)
                .expect("Failed reading the template file");
            stdout_template = template.clone();
            json_formatter = Box::new(move |data: HashMap<String, String>| -> serde_json::Value {
                let result = fill_template(template.clone(), data);
                serde_json::from_str(result.as_str()).unwrap()
            });
        }
        None => {}
    };
    let json_printer = JsonPrinter { json_formatter };

    let stdout_printer = StdoutPrinter {
        output_template: stdout_template,
    };

    let printer: &dyn Printer = match args.output_format {
        OutputFormat::Json => &json_printer,
        OutputFormat::Stdout => &stdout_printer,
        _ => panic!("Unsupported format!"),
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
    printer.write_to_file(data.to_dict(), output_file_path_str);
}
