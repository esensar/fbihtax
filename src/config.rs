extern crate serde;
extern crate serde_json;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{fs::File, io::BufReader};

use crate::error::{Error, Result};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default)]
    pub pdf: PdfConfig,
    #[serde(default)]
    pub ams: AmsConfig,
    #[serde(default)]
    pub gpd: GpdConfig,
    #[serde(default = "default_output_location")]
    pub output_location: String,
    #[serde(default = "default_db_location")]
    pub db_location: String,
    pub user: Option<UserConfig>,
    pub client: Option<ClientConfig>,
}

fn default_output_location() -> String {
    ".".to_string()
}

fn default_db_location() -> String {
    "fbihtax.db.json".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfConfig {
    #[serde(default = "default_pdftk_path")]
    pub pdftk_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmsConfig {
    #[serde(default = "default_ams_cache_location")]
    pub cache_location: String,
    #[serde(default = "default_ams_download_url")]
    pub download_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GpdConfig {
    #[serde(default = "default_gpd_cache_location")]
    pub cache_location: String,
    #[serde(default = "default_gpd_download_url")]
    pub download_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserConfig {
    pub name: String,
    pub address: String,
    pub jmbg: String,
    pub phone: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClientConfig {
    pub name: String,
    pub address: String,
    pub country: String,
}

fn default_ams_cache_location() -> String {
    "amscache.pdf".to_string()
}

fn default_ams_download_url() -> String {
    "http://www.pufbih.ba/v1/public/upload/obrasci/b839c-obrazac-ams_bos_web.pdf".to_string()
}

fn default_gpd_cache_location() -> String {
    "gpdcache.pdf".to_string()
}

fn default_gpd_download_url() -> String {
    "http://www.pufbih.ba/v1/public/upload/obrasci/a9d63-94b8a-obrazac_gpd_1051_ver1__bos_web2.pdf"
        .to_string()
}

fn default_pdftk_path() -> String {
    "pdftk".to_string()
}

impl Default for PdfConfig {
    fn default() -> Self {
        Self {
            pdftk_path: default_pdftk_path(),
        }
    }
}

impl Default for AmsConfig {
    fn default() -> Self {
        Self {
            cache_location: default_ams_cache_location(),
            download_url: default_ams_download_url(),
        }
    }
}

impl Default for GpdConfig {
    fn default() -> Self {
        Self {
            cache_location: default_gpd_cache_location(),
            download_url: default_gpd_download_url(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            output_location: default_output_location(),
            db_location: default_db_location(),
            pdf: Default::default(),
            ams: AmsConfig::default(),
            gpd: GpdConfig::default(),
            user: None,
            client: None,
        }
    }
}

fn parse_from_reader<T: DeserializeOwned>(reader: BufReader<File>) -> Result<T> {
    serde_json::from_reader(reader).map_err(Error::from)
}

pub fn parse_config<T: for<'de> Deserialize<'de>>(config_location: &str) -> Result<T> {
    parse_from_reader(File::open(config_location).map(BufReader::new)?)
}

pub fn parse_config_with_default<T: Default + for<'de> Deserialize<'de>>(
    config_location: &str,
) -> T {
    parse_config(config_location).unwrap_or_default()
}
