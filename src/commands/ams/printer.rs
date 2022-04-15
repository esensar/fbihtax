extern crate clap;
extern crate rust_decimal;

use std::env::temp_dir;
use std::fs::File;

use crate::{
    config::Config,
    fdf::fdf_generator::{self, FdfData},
    forms::amsform,
};

#[derive(PartialEq, Eq)]
pub enum OutputFormat {
    Pdf,
    Fdf,
    Xfdf,
    Json,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match &self {
            OutputFormat::Pdf => "pdf",
            OutputFormat::Fdf => "fdf",
            OutputFormat::Xfdf => "xfdf",
            OutputFormat::Json => "json",
        })
    }
}

impl std::fmt::Debug for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match &self {
            OutputFormat::Pdf => "pdf",
            OutputFormat::Fdf => "fdf",
            OutputFormat::Xfdf => "xfdf",
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
            "xfdf" => Ok(OutputFormat::Xfdf),
            "json" => Ok(OutputFormat::Json),
            _ => Err("Unknown format passed!".to_string()),
        }
    }
}

pub struct PdfPrinter<'a> {
    pub config: &'a Config,
    pub xfdf_printer: &'a XfdfPrinter,
}
pub struct FdfPrinter {}
pub struct XfdfPrinter {}
pub struct JsonPrinter {}

pub trait AmsPrinter {
    fn write_to_file(&self, form: &mut amsform::AmsForm, file: &str);
}

impl<'a> AmsPrinter for PdfPrinter<'a> {
    fn write_to_file(&self, form: &mut amsform::AmsForm, file: &str) {
        let mut tmp_fdf_file = temp_dir();
        tmp_fdf_file.push("fbihtax.xfdf");
        let tmp_fdf_file_str = tmp_fdf_file.to_str().expect("Can't create temporary file");
        self.xfdf_printer.write_to_file(form, tmp_fdf_file_str);
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

impl AmsPrinter for XfdfPrinter {
    fn write_to_file(&self, form: &mut amsform::AmsForm, file: &str) {
        let dict = form.to_dict();
        let fdf_data = FdfData::from_dict(dict);
        fdf_generator::write_xfdf(fdf_data, file.to_string());
    }
}

impl AmsPrinter for JsonPrinter {
    fn write_to_file(&self, form: &mut amsform::AmsForm, file: &str) {
        let breakdown_writer = File::create(file).expect("Failed creating output JSON");
        let dict = form.to_dict();
        serde_json::to_writer_pretty(breakdown_writer, &dict).expect("Failed saving output JSON");
    }
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
