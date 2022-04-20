extern crate clap;
extern crate rust_decimal;

use std::{fs::File, path::Path};

use crate::{
    config::{self, ClientConfig, Config, UserConfig},
    db::{self, AmsInfo, TaxDb},
    error::{FbihtaxError, FbihtaxResult, UserErrorKind},
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
    #[clap(
        long,
        help = "Invoice date (YYYY-MM-DD). Must be present for DB to work"
    )]
    invoice_date: Option<String>,
    #[clap(
        short,
        long,
        help = "Tax deduction percentage (20 default, 30 for income from authored work)",
        default_value_t = dec!(20)
    )]
    deduction_percentage: Decimal,
    #[clap(long, help = "Output format (PDF, FDF, XFDF, JSON)", default_value_t = OutputFormat::Pdf)]
    output_format: OutputFormat,
    #[clap(long, help = "Path to config file with user specific settings")]
    user_config: Option<String>,
    #[clap(long, help = "Path to config file with client specific settings")]
    client_config: Option<String>,
    #[clap(
        short,
        long,
        help = "Path to save output file to",
        default_value = "amsform.pdf"
    )]
    output: String,
    #[clap(
        long,
        help = "By default DB file is updated with this AMS form. Add this flag to skip writing to db"
    )]
    skip_db: bool,
}

pub fn handle_command(config: Config, args: &AmsArgs) -> FbihtaxResult<()> {
    if !Path::new(config.ams.cache_location.as_str()).exists() {
        println!(
            "Cached AMS form not found at: {}\nResorting to download from: {}",
            config.ams.cache_location, config.ams.download_url,
        );
        let mut result = reqwest::blocking::get(config.ams.download_url.to_string())?;
        let mut file_writer = File::create(config.ams.cache_location.as_str())?;
        result.copy_to(&mut file_writer)?;
        println!(
            "Downloaded AMS form and cached to: {}",
            config.ams.cache_location,
        );
    }

    let mut form = amsform::load_ams_form(config.ams.cache_location.clone())?;

    let fdf_printer = FdfPrinter {};
    let xfdf_printer = XfdfPrinter {};
    let json_printer = JsonPrinter::default();
    let pdf_printer = PdfPrinter {
        config: &config,
        source_pdf: config.ams.cache_location.clone(),
        xfdf_printer: &xfdf_printer,
    };

    let printer: &dyn Printer = match args.output_format {
        OutputFormat::Pdf => &pdf_printer,
        OutputFormat::Fdf => &fdf_printer,
        OutputFormat::Xfdf => &xfdf_printer,
        OutputFormat::Json => &json_printer,
        format => {
            return Err(FbihtaxError::UserError(
                UserErrorKind::UnsupportedOutputFormat(format),
            ))
        }
    };

    let income_dec: Decimal = args.income.round_dp(2);
    let deduction_factor: Decimal = dec!(1) - (args.deduction_percentage.round_dp(2) * dec!(0.01));
    let income_after = income_dec * deduction_factor;

    let user_config = match &args.user_config {
        Some(path) => config::parse_config::<UserConfig>(path.as_str())?,
        None => {
            config
                .user
                .clone()
                .ok_or(FbihtaxError::UserError(UserErrorKind::MissingConfig(
                    "user configuration".to_string(),
                    "--user-config".to_string(),
                )))?
        }
    };
    form.fill_main_field(FormField::UserName, user_config.name)?;
    form.fill_main_field(FormField::UserAddress, user_config.address)?;
    form.fill_main_field(FormField::UserJmbg, user_config.jmbg)?;

    if let Some(invoice_date) = &args.invoice_date {
        if let Some((year, rest)) = invoice_date.split_once("-") {
            if let Some((month, day)) = rest.split_once("-") {
                let year_last_2 = &year[2..year.len()];
                form.fill_main_field(FormField::TaxPeriodMonth, month.to_string())?;
                form.fill_main_field(FormField::TaxPeriodYearLast2Digits, year_last_2.to_string())?;
                form.fill_main_field(FormField::PaymentDateDay, day.to_string())?;
                form.fill_main_field(FormField::PaymentDateMonth, month.to_string())?;
                form.fill_main_field(FormField::PaymentDateYear, year_last_2.to_string())?;
            }
        }
    }

    let client_config =
        match &args.user_config {
            Some(path) => config::parse_config::<ClientConfig>(path.as_str())?,
            None => config.client.clone().ok_or(FbihtaxError::UserError(
                UserErrorKind::MissingConfig(
                    "client configuration".to_string(),
                    "--client-config".to_string(),
                ),
            ))?,
        };
    form.fill_main_field(FormField::CompanyName, client_config.name)?;
    form.fill_main_field(FormField::CompanyAddress, client_config.address)?;
    form.fill_main_field(FormField::CompanyCountry, client_config.country)?;

    let ams_info = form.add_income(income_after, dec!(0));

    let output_path = Path::new(config.output_location.as_str());
    let mut output_file_path = output_path.join(args.output.clone());
    let extension = format!("{}", args.output_format);
    output_file_path.set_extension(extension);
    let output_file_path_str =
        output_file_path
            .to_str()
            .ok_or(FbihtaxError::UserError(UserErrorKind::Generic(
                "Output location seems to be invalid!".to_string(),
            )))?;

    printer.write_to_file(form.to_dict()?, output_file_path_str)?;
    println!("Saved AMS form to: {}", output_file_path_str);

    if !args.skip_db {
        if let Some(invoice_date) = &args.invoice_date {
            write_to_db(&config, ams_info, invoice_date.clone())?;
        }
    }
    Ok(())
}

fn write_to_db(config: &Config, ams_info: AmsInfo, invoice_date: String) -> FbihtaxResult<()> {
    println!("Loading database file");
    let mut tax_db: TaxDb = db::parse_db_with_default(config.db_location.as_str());
    tax_db.add_ams_info(ams_info, invoice_date);
    tax_db.write_to_file(config.db_location.as_str())?;
    println!(
        "Successfully updated DB file: {}",
        config.db_location.as_str(),
    );
    Ok(())
}
