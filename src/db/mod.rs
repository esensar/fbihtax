use std::{collections::HashMap, fs::File, io::BufReader};

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxDb {
    #[serde(default = "default_ams_map")]
    pub ams: HashMap<String, AmsInfo>,
}

fn default_ams_map() -> HashMap<String, AmsInfo> {
    return HashMap::new();
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmsInfo {
    pub income_total: Decimal,
    pub tax_paid: Decimal,
}

impl Default for TaxDb {
    fn default() -> Self {
        Self {
            ams: default_ams_map(),
        }
    }
}

impl TaxDb {
    pub fn total_income_for_year(&self, year: String) -> Decimal {
        let mut total = dec!(0);
        for (k, v) in &self.ams {
            if k.starts_with(&year) {
                total += v.income_total;
            }
        }
        return total;
    }

    pub fn total_tax_paid_for_year(&self, year: String) -> Decimal {
        let mut total = dec!(0);
        for (k, v) in &self.ams {
            if k.starts_with(&year) {
                total += v.tax_paid;
            }
        }
        return total;
    }

    pub fn add_ams_info(&mut self, ams_info: AmsInfo, invoice_date: String) {
        self.ams.insert(invoice_date, ams_info);
    }

    pub fn write_to_file(&self, file: &str) {
        let breakdown_writer = File::create(file).expect("Failed creating output JSON");
        serde_json::to_writer_pretty(breakdown_writer, &self).expect("Failed saving output JSON");
    }
}

fn parse_from_reader<T: DeserializeOwned>(reader: BufReader<File>) -> Result<T, String> {
    serde_json::from_reader(reader).map_err(|err| err.to_string())
}

fn parse_db_to_result<T: for<'de> Deserialize<'de>>(config_location: &str) -> Result<T, String> {
    File::open(config_location)
        .map(BufReader::new)
        .map_err(|err| err.to_string())
        .and_then(parse_from_reader)
}

pub fn parse_db_with_default<T: Default + for<'de> Deserialize<'de>>(config_location: &str) -> T {
    parse_db_to_result(config_location).unwrap_or_default()
}

pub fn parse_db<T: for<'de> Deserialize<'de>>(db_location: &str) -> Option<T> {
    match parse_db_to_result(db_location) {
        Ok(r) => Some(r),
        Err(_) => None,
    }
}
