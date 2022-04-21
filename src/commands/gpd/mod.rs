extern crate clap;
extern crate rust_decimal;

use std::{fs::File, path::Path};

use crate::{
    config::{self, Config, UserConfig},
    db::{self, TaxDb},
    error::{self, Error, UserErrorKind},
    format::printer::{FdfPrinter, JsonPrinter, PdfPrinter, Printer, XfdfPrinter},
    format::OutputFormat,
    forms::gpdform::{self, FormField},
};
use clap::Parser;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Parser, Debug)]
pub struct GpdArgs {
    #[clap(long, help = "Year (YYYY)")]
    year: String,
    #[clap(
        long,
        help = "Personal deduction (by default it is 300 for each months => 300 * 12 = 3600)",
        default_value_t = dec!(3600)
    )]
    personal_deduction: Decimal,
    #[clap(
        long,
        help = "Sum of 11th column of GIP form (tax base)",
        default_value_t = dec!(0)
    )]
    gip_income: Decimal,
    #[clap(
        long,
        help = "Sum of 15th column of GIP form (taxes paid)",
        default_value_t = dec!(0)
    )]
    gip_tax_paid: Decimal,
    #[clap(long, help = "Output format (PDF, FDF, XFDF, JSON)", default_value_t = OutputFormat::Pdf)]
    output_format: OutputFormat,
    #[clap(long, help = "Path to config file with user specific settings")]
    user_config: Option<String>,
    #[clap(
        short,
        long,
        help = "Path to save output file to",
        default_value = "gpdform.pdf"
    )]
    output: String,
}

pub fn handle_command(config: Config, args: &GpdArgs) -> error::Result<()> {
    if !Path::new(config.gpd.cache_location.as_str()).exists() {
        println!(
            "Cached GPD form not found at: {}\nResorting to download from: {}",
            config.gpd.cache_location, config.gpd.download_url
        );
        let mut result = reqwest::blocking::get(config.gpd.download_url.to_string())?;
        let mut file_writer = File::create(config.gpd.cache_location.as_str())?;
        result.copy_to(&mut file_writer)?;
        println!(
            "Downloaded GPD form and cached to: {}",
            config.gpd.cache_location
        );
    }

    let mut form = gpdform::load_gpd_form(config.gpd.cache_location.clone())?;
    let db: TaxDb = db::parse_db_with_default(config.db_location.as_str());

    let fdf_printer = FdfPrinter {};
    let xfdf_printer = XfdfPrinter {};
    let json_printer = JsonPrinter::default();
    let pdf_printer = PdfPrinter {
        config: &config,
        source_pdf: config.gpd.cache_location.clone(),
        xfdf_printer: &xfdf_printer,
    };

    let printer: &dyn Printer = match args.output_format {
        OutputFormat::Pdf => &pdf_printer,
        OutputFormat::Fdf => &fdf_printer,
        OutputFormat::Xfdf => &xfdf_printer,
        OutputFormat::Json => &json_printer,
        format => {
            return Err(Error::UserError(UserErrorKind::UnsupportedOutputFormat(
                format,
            )))
        }
    };

    let user_config = match &args.user_config {
        Some(path) => config::parse_config::<UserConfig>(path.as_str())?,
        None => config
            .user
            .clone()
            .ok_or(Error::UserError(UserErrorKind::MissingConfig(
                "user configuration".to_string(),
                "--user-config".to_string(),
            )))?,
    };
    form.fill_user_info(&user_config)?;
    form.fill_year_info(args.year.clone())?;
    form.fill_field(
        FormField::PersonalDeduction,
        args.personal_deduction.to_string(),
    )?;
    form.add_gip_info(args.gip_income, args.gip_tax_paid);
    form.add_deductions(args.personal_deduction, dec!(0), dec!(0));
    form.add_ams_info(
        db.total_income_for_year(args.year.clone()),
        db.total_tax_paid_for_year(args.year.clone()),
    );

    let output_path = Path::new(config.output_location.as_str());
    let mut output_file_path = output_path.join(args.output.clone());
    let extension = format!("{}", args.output_format);
    output_file_path.set_extension(extension);
    let output_file_path_str =
        output_file_path
            .to_str()
            .ok_or(Error::UserError(UserErrorKind::Generic(
                "Output location seems to be invalid!".to_string(),
            )))?;

    printer.write_to_file(form.to_dict()?, output_file_path_str)?;
    println!("Saved GPD form to: {}", output_file_path_str);
    Ok(())
}
