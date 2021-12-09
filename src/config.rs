extern crate serde;
extern crate serde_json;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufReader};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default)]
    pub pdf: PdfConfig,
    #[serde(default = "default_output_location")]
    pub output_location: String,
}

fn default_output_location() -> String {
    ".".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfConfig {
    #[serde(default = "default_cache_location")]
    pub cache_location: String,
    #[serde(default = "default_download_url")]
    pub download_url: String,
}

fn default_cache_location() -> String {
    "tax.pdf".to_string()
}

fn default_download_url() -> String {
    "http://www.pufbih.ba/v1/public/upload/obrasci/b839c-obrazac-ams_bos_web.pdf".to_string()
}

impl Default for PdfConfig {
    fn default() -> Self {
        Self {
            cache_location: default_cache_location(),
            download_url: default_download_url(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            output_location: default_output_location(),
            ..Default::default()
        }
    }
}

fn parse_from_reader(reader: BufReader<File>) -> Result<Config, String> {
    serde_json::from_reader(reader).map_err(|err| err.to_string())
}

pub fn parse_config(config_location: &str) -> Config {
    File::open(config_location)
        .map(BufReader::new)
        .map_err(|err| err.to_string())
        .and_then(parse_from_reader)
        .unwrap_or_default()
}
