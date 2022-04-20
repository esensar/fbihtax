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
    error::{FbihtaxError, FbihtaxResult, UserErrorKind},
    format::printer::{JsonPrinter, Printer, StdoutPrinter},
    format::{utils::fill_template, OutputFormat},
    taxcalculator,
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

fn default_json_formatter(data: HashMap<String, String>) -> FbihtaxResult<serde_json::Value> {
    Ok(json!({
        "income_tax": data.get("income_tax"),
        "health_insurance": {
            "federation": data.get("health_insurance_federation"),
            "canton": data.get("health_insurance_canton"),
            "total": data.get("health_insurance_total")
        },
        "total": data.get("total")
    }))
}

pub fn handle_command(config: Config, args: &TaxBreakdownArgs) -> FbihtaxResult<()> {
    let mut json_formatter: Box<
        dyn Fn(HashMap<String, String>) -> FbihtaxResult<serde_json::Value>,
    > = Box::new(default_json_formatter);

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
            json_formatter = Box::new(
                move |data: HashMap<String, String>| -> FbihtaxResult<serde_json::Value> {
                    let result = fill_template(template.clone(), data);
                    serde_json::from_str(result.as_str()).map_err(FbihtaxError::from)
                },
            );
        }
        None => {}
    };
    match args.output_template_file.clone() {
        Some(template_file_path) => {
            let mut template_file = File::open(template_file_path.clone())?;
            let mut template = String::new();
            template_file.read_to_string(&mut template)?;
            stdout_template = template.clone();
            json_formatter = Box::new(
                move |data: HashMap<String, String>| -> FbihtaxResult<serde_json::Value> {
                    let result = fill_template(template.clone(), data);
                    serde_json::from_str(result.as_str()).map_err(FbihtaxError::from)
                },
            );
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
        format => {
            return Err(FbihtaxError::UserError(
                UserErrorKind::UnsupportedOutputFormat(format),
            ))
        }
    };

    let deduced_income = taxcalculator::income_after_deduction(
        args.income.round_dp(2),
        args.deduction_percentage.round_dp(2),
    );

    let output_path = Path::new(config.output_location.as_str());
    let output_file_path = output_path.join(&args.output);
    let output_file_path_str =
        output_file_path
            .to_str()
            .ok_or(FbihtaxError::UserError(UserErrorKind::Generic(
                "Output location seems to be invalid!".to_string(),
            )))?;
    let data = TaxBreakdownData {
        income_tax: taxcalculator::tax_amount(deduced_income),
        health_insurance_federation: taxcalculator::health_insurance_federation(deduced_income),
        health_insurance_canton: taxcalculator::health_insurance_canton(deduced_income),
    };
    printer.write_to_file(data.to_dict(), output_file_path_str)
}
