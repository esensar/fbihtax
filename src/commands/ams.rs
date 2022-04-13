extern crate clap;
extern crate rust_decimal;

use std::fs::File;
use std::{env::temp_dir, path::Path};

use clap::Parser;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::{
    config::{self, ClientConfig, Config, UserConfig},
    fdf::fdf_generator::{self, FdfData},
    forms::amsform::{self, FormField},
};

#[derive(PartialEq, Eq)]
pub enum OutputFormat {
    Pdf,
    Fdf,
    Json,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match &self {
            OutputFormat::Pdf => "pdf",
            OutputFormat::Fdf => "fdf",
            OutputFormat::Json => "json",
        })
    }
}

impl std::fmt::Debug for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match &self {
            OutputFormat::Pdf => "pdf",
            OutputFormat::Fdf => "fdf",
            OutputFormat::Json => "json",
        })
    }
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pdf" => Ok(OutputFormat::Pdf),
            "fdf" => Ok(OutputFormat::Fdf),
            "json" => Ok(OutputFormat::Json),
            _ => Err("Unknown format passed!".to_string()),
        }
    }
}

struct PdfPrinter<'a> {
    config: &'a Config,
    fdf_printer: &'a FdfPrinter,
}
struct FdfPrinter {}
struct JsonPrinter {}

trait AmsPrinter {
    fn write_to_file(&self, form: &mut amsform::AmsForm, file: &str);
}

impl<'a> AmsPrinter for PdfPrinter<'a> {
    fn write_to_file(&self, form: &mut amsform::AmsForm, file: &str) {
        let mut tmp_fdf_file = temp_dir();
        tmp_fdf_file.push("fbihtax.fdf");
        let tmp_fdf_file_str = tmp_fdf_file.to_str().expect("Can't create temporary file");
        self.fdf_printer.write_to_file(form, tmp_fdf_file_str);
        let _process = std::process::Command::new(&self.config.pdf.pdftk_path)
            .args(&[
                self.config.pdf.cache_location.clone(),
                "fill_form".to_string(),
                tmp_fdf_file_str.to_string(),
                "output".to_string(),
                file.to_string()
            ])
            .output()
            .ok()
            .expect("Failed to execute pdftk. Ensure it is installed and path is properly configured in .fbihtax.json");
    }
}

impl AmsPrinter for FdfPrinter {
    fn write_to_file(&self, form: &mut amsform::AmsForm, file: &str) {
        let dict = form.to_dict();
        let fdf_data = FdfData::from_dict(dict);
        fdf_generator::write_fdf(fdf_data, file.to_string());
    }
}

impl AmsPrinter for JsonPrinter {
    fn write_to_file(&self, form: &mut amsform::AmsForm, file: &str) {
        let breakdown_writer = File::create(file).expect("Failed creating output JSON");
        let dict = form.to_dict();
        serde_json::to_writer_pretty(breakdown_writer, &dict).expect("Failed saving output JSON");
    }
}

#[derive(Parser, Debug)]
pub struct AmsArgs {
    #[clap(
        short,
        long,
        help = "Decimal income value in BAM (will be rounded to 2 decimals)"
    )]
    income: Decimal,
    #[clap(long, help = "Invoice date (YYYY-MM-DD)")]
    invoice_date: Option<String>,
    #[clap(
        short,
        long,
        help = "Tax deduction percentage (20 default, 30 for income from authored work)",
        default_value_t = dec!(20)
    )]
    deduction_percentage: Decimal,
    #[clap(long, help = "Output format (PDF, FDF, JSON)", default_value_t = OutputFormat::Json)]
    output_format: OutputFormat,
    #[clap(long, help = "Path to config file with user specific settings")]
    user_config: Option<String>,
    #[clap(long, help = "Path to config file with client specific settings")]
    client_config: Option<String>,
    #[clap(
        short,
        long,
        help = "Path to save output file to",
        default_value = "amsform.json"
    )]
    output: String,
}

pub fn handle_command(config: Config, args: &AmsArgs) {
    let mut form = amsform::load_ams_form(config.pdf.cache_location.clone());

    let fdf_printer = FdfPrinter {};
    let json_printer = JsonPrinter {};
    let pdf_printer = PdfPrinter {
        config: &config,
        fdf_printer: &fdf_printer,
    };

    let printer: &dyn AmsPrinter = match args.output_format {
        OutputFormat::Pdf => &pdf_printer,
        OutputFormat::Fdf => &fdf_printer,
        OutputFormat::Json => &json_printer,
    };

    let income_dec: Decimal = args.income.round_dp(2);
    let deduction_factor: Decimal = dec!(1) - (args.deduction_percentage.round_dp(2) * dec!(0.01));
    let income_after = income_dec * deduction_factor;

    let user_config = match &args.user_config {
        Some(path) => config::parse_config::<UserConfig>(path.as_str()),
        None => match &config.user {
            Some(user_config) => Some(user_config.clone()),
            None => None
        }
    }.expect("Missing user configuration. Either fill it in default config file or pass --user-config parameter.");
    form.fill_main_field(FormField::UserName, user_config.name);
    form.fill_main_field(FormField::UserAddress, user_config.address);
    form.fill_main_field(FormField::UserJmbg, user_config.jmbg);

    if let Some(invoice_date) = &args.invoice_date {
        if let Some((year, rest)) = invoice_date.split_once("-") {
            if let Some((month, day)) = rest.split_once("-") {
                let year_last_2 = &year[2..year.len()];
                form.fill_main_field(FormField::TaxPeriodMonth, month.to_string());
                form.fill_main_field(FormField::TaxPeriodYearLast2Digits, year_last_2.to_string());
                form.fill_main_field(FormField::PaymentDateDay, day.to_string());
                form.fill_main_field(FormField::PaymentDateMonth, month.to_string());
                form.fill_main_field(FormField::PaymentDateYear, year_last_2.to_string());
            }
        }
    }

    let client_config = match &args.client_config {
        Some(path) => config::parse_config::<ClientConfig>(path.as_str()),
        None => match &config.client {
            Some(client_config) => Some(client_config.clone()),
            None => None
        }
    }.expect("Missing client configuration. Either fill it in default config file or pass --client-config parameter.");
    form.fill_main_field(FormField::CompanyName, client_config.name);
    form.fill_main_field(FormField::CompanyAddress, client_config.address);
    form.fill_main_field(FormField::CompanyCountry, client_config.country);

    form.add_income(income_after, dec!(0));

    let output_path = Path::new(config.output_location.as_str());
    let mut output_file_path = output_path.join(args.output.clone());
    let extension = format!("{}", args.output_format);
    output_file_path.set_extension(extension);
    let output_file_path_str = output_file_path
        .to_str()
        .expect("Output location seems to be invalid!");

    printer.write_to_file(&mut form, output_file_path_str);
    println!("Saved AMS form to: {}", output_file_path_str);
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn output_format_name_normal_test() {
        assert_eq!(OutputFormat::Pdf, OutputFormat::from_str("pdf").unwrap());
        assert_eq!(OutputFormat::Json, OutputFormat::from_str("json").unwrap());
        assert_eq!(OutputFormat::Fdf, OutputFormat::from_str("fdf").unwrap());
    }

    #[test]
    fn output_format_name_unsupported_test() {
        assert!(OutputFormat::from_str("yml").is_err());
        assert!(OutputFormat::from_str("xml").is_err());
    }

    #[test]
    fn output_format_name_wrong_case_test() {
        assert!(OutputFormat::from_str("PDF").is_err());
        assert!(OutputFormat::from_str("JSON").is_err());
        assert!(OutputFormat::from_str("FDF").is_err());
    }

    #[test]
    fn output_format_display_test() {
        assert_eq!("format is pdf", format!("format is {}", OutputFormat::Pdf));
        assert_eq!(
            "format is json",
            format!("format is {}", OutputFormat::Json)
        );
        assert_eq!("format is fdf", format!("format is {}", OutputFormat::Fdf));
    }
}
