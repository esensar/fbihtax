extern crate serde;
extern crate serde_json;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{fs::File, io::BufReader};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default)]
    pub pdf: PdfConfig,
    #[serde(default = "default_output_location")]
    pub output_location: String,
    pub user: Option<UserConfig>,
    pub client: Option<ClientConfig>,
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
    #[serde(default = "default_pdftk_path")]
    pub pdftk_path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserConfig {
    pub name: String,
    pub address: String,
    pub jmbg: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClientConfig {
    pub name: String,
    pub address: String,
    pub country: String,
}

fn default_cache_location() -> String {
    "tax.pdf".to_string()
}

fn default_download_url() -> String {
    "http://www.pufbih.ba/v1/public/upload/obrasci/b839c-obrazac-ams_bos_web.pdf".to_string()
}

fn default_pdftk_path() -> String {
    "pdftk".to_string()
}

impl Default for PdfConfig {
    fn default() -> Self {
        Self {
            cache_location: default_cache_location(),
            download_url: default_download_url(),
            pdftk_path: default_pdftk_path(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            output_location: default_output_location(),
            pdf: Default::default(),
            user: None,
            client: None,
        }
    }
}

fn parse_from_reader<T: DeserializeOwned>(reader: BufReader<File>) -> Result<T, String> {
    serde_json::from_reader(reader).map_err(|err| err.to_string())
}

fn parse_config_to_result<T: for<'de> Deserialize<'de>>(
    config_location: &str,
) -> Result<T, String> {
    File::open(config_location)
        .map(BufReader::new)
        .map_err(|err| err.to_string())
        .and_then(parse_from_reader)
}

pub fn parse_config_with_default<T: Default + for<'de> Deserialize<'de>>(
    config_location: &str,
) -> T {
    parse_config_to_result(config_location).unwrap_or_default()
}

pub fn parse_config<T: for<'de> Deserialize<'de>>(config_location: &str) -> Option<T> {
    match parse_config_to_result(config_location) {
        Ok(r) => Some(r),
        Err(_) => None,
    }
}
