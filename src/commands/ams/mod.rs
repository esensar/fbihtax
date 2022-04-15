extern crate clap;
extern crate rust_decimal;

use std::path::Path;

use crate::{
    config::{self, ClientConfig, Config, UserConfig},
    format::printer::{FdfPrinter, JsonPrinter, PdfPrinter, Printer, XfdfPrinter},
    format::OutputFormat,
    forms::amsform::{self, FormField},
};
use clap::Parser;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

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
    #[clap(long, help = "Output format (PDF, FDF, XFDF, JSON)", default_value_t = OutputFormat::Json)]
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
    let xfdf_printer = XfdfPrinter {};
    let json_printer = JsonPrinter::default();
    let pdf_printer = PdfPrinter {
        config: &config,
        source_pdf: config.pdf.cache_location.clone(),
        xfdf_printer: &xfdf_printer,
    };

    let printer: &dyn Printer = match args.output_format {
        OutputFormat::Pdf => &pdf_printer,
        OutputFormat::Fdf => &fdf_printer,
        OutputFormat::Xfdf => &xfdf_printer,
        OutputFormat::Json => &json_printer,
        _ => panic!("Unsupported format!"),
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

    printer.write_to_file(form.to_dict(), output_file_path_str);
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
        assert_eq!(OutputFormat::Xfdf, OutputFormat::from_str("xfdf").unwrap());
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
        assert!(OutputFormat::from_str("XFDF").is_err());
    }

    #[test]
    fn output_format_display_test() {
        assert_eq!("format is pdf", format!("format is {}", OutputFormat::Pdf));
        assert_eq!(
            "format is json",
            format!("format is {}", OutputFormat::Json)
        );
        assert_eq!("format is fdf", format!("format is {}", OutputFormat::Fdf));
        assert_eq!(
            "format is xfdf",
            format!("format is {}", OutputFormat::Xfdf)
        );
    }
}