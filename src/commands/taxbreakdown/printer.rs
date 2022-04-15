extern crate clap;
extern crate rust_decimal;

use std::fs::File;

use rust_decimal::Decimal;
use serde_json::json;

#[derive(PartialEq, Eq)]
pub enum OutputFormat {
    Json,
    Stdout,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match &self {
            OutputFormat::Json => "json",
            OutputFormat::Stdout => "stdout",
        })
    }
}

impl std::fmt::Debug for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match &self {
            OutputFormat::Json => "json",
            OutputFormat::Stdout => "stdout",
        })
    }
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "json" => Ok(OutputFormat::Json),
            "stdout" => Ok(OutputFormat::Stdout),
            _ => Err("Unknown format passed!".to_string()),
        }
    }
}

pub struct TaxBreakdownData {
    pub income_tax: Decimal,
    pub health_insurance_federation: Decimal,
    pub health_insurance_canton: Decimal,
}

impl TaxBreakdownData {
    fn get_health_insurance_total(&self) -> Decimal {
        return self.health_insurance_federation + self.health_insurance_canton;
    }

    fn get_total(&self) -> Decimal {
        return self.income_tax + self.get_health_insurance_total();
    }
}

pub trait TaxBreakdownPrinter {
    fn write_to_file(&self, data: &TaxBreakdownData, file: &str);
}

pub struct JsonPrinter {}
pub struct StdoutPrinter {}

impl TaxBreakdownPrinter for JsonPrinter {
    fn write_to_file(&self, data: &TaxBreakdownData, file: &str) {
        let breakdown_writer = File::create(file).expect("Failed creating tax breakdown file");
        let json = json!({
            "income_tax": data.income_tax.round_dp(2),
            "health_insurance": {
                "federation": data.health_insurance_federation.round_dp(2),
                "canton": data.health_insurance_canton.round_dp(2),
                "total": data.get_health_insurance_total().round_dp(2)
            },
            "total": data.get_total().round_dp(2)
        });
        serde_json::to_writer_pretty(breakdown_writer, &json).expect("Failed saving tax breakdown");
        println!("Saved tax breakdown to: {}", file);
    }
}

impl TaxBreakdownPrinter for StdoutPrinter {
    fn write_to_file(&self, data: &TaxBreakdownData, file: &str) {
        println!(
            concat!(
                "Income Tax breakdown:\n",
                "\n",
                "Income tax: {income_tax}\n",
                "\n",
                "Health insurance:\n",
                "  Federation: {health_federation}\n",
                "  Canton: {health_canton}\n",
                "  Total: {health_total}\n",
                "\n",
                "Total: {total}\n"
            ),
            income_tax = data.income_tax.round_dp(2),
            health_federation = data.health_insurance_federation.round_dp(2),
            health_canton = data.health_insurance_canton.round_dp(2),
            health_total = data.get_health_insurance_total().round_dp(2),
            total = data.get_total().round_dp(2)
        );
    }
}
