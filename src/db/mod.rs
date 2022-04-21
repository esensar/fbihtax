use std::{collections::HashMap, fs::File, io::BufReader};

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::error::{Error, Result};

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

    pub fn write_to_file(&self, file: &str) -> Result<()> {
        let breakdown_writer = File::create(file)?;
        serde_json::to_writer_pretty(breakdown_writer, &self).map_err(Error::from)
    }
}

fn parse_from_reader<T: DeserializeOwned>(reader: BufReader<File>) -> Result<T> {
    serde_json::from_reader(reader).map_err(Error::from)
}

pub fn parse_db<T: for<'de> Deserialize<'de>>(db_location: &str) -> Result<T> {
    parse_from_reader(File::open(db_location).map(BufReader::new)?)
}

pub fn parse_db_with_default<T: Default + for<'de> Deserialize<'de>>(db_location: &str) -> T {
    parse_db(db_location).unwrap_or_default()
}
