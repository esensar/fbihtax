extern crate clap;
extern crate rust_decimal;

use std::collections::HashMap;

use rust_decimal::Decimal;

pub struct TaxBreakdownData {
    pub income_tax: Decimal,
    pub health_insurance_federation: Decimal,
    pub health_insurance_canton: Decimal,
}

impl TaxBreakdownData {
    fn get_health_insurance_total(&self) -> Decimal {
        return self.health_insurance_federation + self.health_insurance_canton;
    }

    fn get_total(&self) -> Decimal {
        return self.income_tax + self.get_health_insurance_total();
    }

    pub fn to_dict(&self) -> HashMap<String, String> {
        let mut result: HashMap<String, String> = HashMap::new();
        result.insert(
            "income_tax".to_string(),
            self.income_tax.round_dp(2).to_string(),
        );
        result.insert(
            "health_insurance_federation".to_string(),
            self.health_insurance_federation.round_dp(2).to_string(),
        );
        result.insert(
            "health_insurance_canton".to_string(),
            self.health_insurance_canton.round_dp(2).to_string(),
        );
        result.insert(
            "health_insurance_total".to_string(),
            self.get_health_insurance_total().round_dp(2).to_string(),
        );
        result.insert(
            "total".to_string(),
            self.get_total().round_dp(2).to_string(),
        );
        return result;
    }
}
