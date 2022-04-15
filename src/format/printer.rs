use std::{
    collections::HashMap,
    env::temp_dir,
    fs::File,
    io::{self, Write},
};

use serde_json::json;

use crate::{
    config::Config,
    fdf::fdf_generator::{self, FdfData},
};

use super::utils::fill_template;

pub trait Printer {
    fn write_to_file(&self, data: HashMap<String, String>, file: &str);
}

pub struct PdfPrinter<'a> {
    pub config: &'a Config,
    pub source_pdf: String,
    pub xfdf_printer: &'a XfdfPrinter,
}
pub struct FdfPrinter {}
pub struct XfdfPrinter {}
pub struct JsonPrinter {
    pub json_formatter: Box<dyn Fn(HashMap<String, String>) -> serde_json::Value>,
}
pub struct StdoutPrinter {
    pub output_template: String,
}

fn default_json_formatter(data: HashMap<String, String>) -> serde_json::Value {
    json!(data)
}

impl Default for JsonPrinter {
    fn default() -> Self {
        return Self {
            json_formatter: Box::new(default_json_formatter),
        };
    }
}
impl<'a> Printer for PdfPrinter<'a> {
    fn write_to_file(&self, data: HashMap<String, String>, file: &str) {
        let mut tmp_fdf_file = temp_dir();
        tmp_fdf_file.push("fbihtax.xfdf");
        let tmp_fdf_file_str = tmp_fdf_file.to_str().expect("Can't create temporary file");
        self.xfdf_printer.write_to_file(data, tmp_fdf_file_str);
        let _process = std::process::Command::new(&self.config.pdf.pdftk_path)
            .args(&[
                self.source_pdf.clone(),
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

impl Printer for FdfPrinter {
    fn write_to_file(&self, data: HashMap<String, String>, file: &str) {
        let fdf_data = FdfData::from_dict(data);
        fdf_generator::write_fdf(fdf_data, file.to_string());
    }
}

impl Printer for XfdfPrinter {
    fn write_to_file(&self, data: HashMap<String, String>, file: &str) {
        let fdf_data = FdfData::from_dict(data);
        fdf_generator::write_xfdf(fdf_data, file.to_string());
    }
}

impl Printer for JsonPrinter {
    fn write_to_file(&self, data: HashMap<String, String>, file: &str) {
        let breakdown_writer = File::create(file).expect("Failed creating output JSON");
        let result_json = (self.json_formatter)(data);
        serde_json::to_writer_pretty(breakdown_writer, &result_json)
            .expect("Failed saving output JSON");
    }
}

impl Printer for StdoutPrinter {
    fn write_to_file(&self, data: HashMap<String, String>, _file: &str) {
        let result = fill_template(self.output_template.clone(), data);
        io::stdout().write(result.as_bytes()).unwrap();
    }
}
