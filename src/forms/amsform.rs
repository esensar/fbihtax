extern crate pdf_forms;
extern crate rust_decimal;
use std::{collections::HashMap, ops::Add};

use crate::{
    db::AmsInfo,
    error::{Error, Result},
    forms::formutils::{fill_field, format_money_value},
    taxcalculator,
};
use pdf_forms::Form;
use rust_decimal::Decimal;

#[derive(Clone, Copy)]
pub enum FormField {
    PageNumber = 0,
    PageCount = 1,
    UserName = 2,
    UserJmbg = 3,
    UserAddress = 4,
    PaymentDateDay = 5,
    PaymentDateMonth = 6,
    PaymentDateYear = 7,
    TaxPeriodMonth = 8,
    TaxPeriodYearLast2Digits = 9,
    CompanyName = 10,
    CompanyAddress = 11,
    CompanyCountry = 12,
    HealthInsuranceTotal = 43,
    TaxBaseTotal = 44,
    TaxAmountTotal = 45,
    TaxPairAbroadTotal = 46,
    TaxToPayTotal = 47,
    Date = 48,
}

#[derive(Clone, Copy)]
enum RepeatingFormField {
    IncomeValue = 0,
    HealthInsurance = 1,
    TaxBase = 2,
    TaxAmount = 3,
    TaxPaidAbroad = 4,
    TaxToPay = 5,
}

static REPEATING_FIELDS_START: u32 = 13;
static REPEATED_LINES: u32 = 5;
static REPEATED_FIELDS_COUNT: u32 = 6;

struct IncomeLine {
    value: Decimal,
    health_insurance: Decimal,
    tax_base: Decimal,
    tax_amount: Decimal,
    tax_paid_abroad: Decimal,
    tax_to_pay: Decimal,
}

impl Add for IncomeLine {
    type Output = IncomeLine;

    fn add(self, rhs: IncomeLine) -> Self::Output {
        IncomeLine {
            value: self.value + rhs.value,
            health_insurance: self.health_insurance + rhs.health_insurance,
            tax_base: self.tax_base + rhs.tax_base,
            tax_amount: self.tax_amount + rhs.tax_amount,
            tax_paid_abroad: self.tax_paid_abroad + rhs.tax_paid_abroad,
            tax_to_pay: self.tax_to_pay + rhs.tax_to_pay,
        }
    }
}

impl Clone for IncomeLine {
    fn clone_from(&mut self, source: &Self) {
        *self = source.clone()
    }

    fn clone(&self) -> Self {
        Self {
            value: self.value,
            health_insurance: self.health_insurance,
            tax_base: self.tax_base,
            tax_amount: self.tax_amount,
            tax_paid_abroad: self.tax_paid_abroad,
            tax_to_pay: self.tax_to_pay,
        }
    }
}
impl Copy for IncomeLine {}

pub struct AmsForm {
    pdf_form: Form,
    fields: HashMap<usize, String>,
    income_lines: Vec<IncomeLine>,
}

fn fill_repeating_field(
    pdf_form: &mut Form,
    line: u32,
    field: RepeatingFormField,
    value: String,
) -> Result<()> {
    fill_field(
        pdf_form,
        (field as u32 + REPEATING_FIELDS_START + line * REPEATED_FIELDS_COUNT)
            .try_into()
            .unwrap(),
        value,
    )
}

impl AmsForm {
    pub fn fill_main_field(&mut self, field: FormField, value: String) -> Result<()> {
        self.fields.insert(field as usize, value.clone());
        fill_field(&mut self.pdf_form, field as usize, value)
    }

    fn fill_repeating_field(
        &mut self,
        line: u32,
        field: RepeatingFormField,
        value: String,
    ) -> Result<()> {
        let field_index = field as u32 + REPEATING_FIELDS_START + line * REPEATED_FIELDS_COUNT;
        self.fields.insert(field_index as usize, value.clone());
        fill_repeating_field(&mut self.pdf_form, line, field, value)
    }

    pub fn add_income(&mut self, base_value: Decimal, tax_paid_abroad: Decimal) -> AmsInfo {
        let health_insurance = taxcalculator::health_insurance_part(base_value);
        let tax_base = taxcalculator::tax_base(base_value);
        let tax_amount = taxcalculator::tax_amount(base_value);
        let tax_to_pay = tax_amount - tax_paid_abroad;
        let income_line = IncomeLine {
            value: base_value,
            health_insurance,
            tax_base,
            tax_amount,
            tax_paid_abroad,
            tax_to_pay,
        };
        self.income_lines.push(income_line);
        return AmsInfo {
            income_total: base_value,
            tax_paid: tax_to_pay,
        };
    }

    fn fill_income_lines(&mut self) -> Result<()> {
        // TODO: Handle multiple pages
        self.fill_main_field(FormField::PageNumber, "1".to_string())?;
        self.fill_main_field(FormField::PageCount, "1".to_string())?;
        let total = self
            .income_lines
            .iter()
            .copied()
            .reduce(|acc, x| acc + x)
            .unwrap();
        let mut counter = 0;
        for income_line in self.income_lines.clone() {
            self.fill_repeating_field(
                counter,
                RepeatingFormField::IncomeValue,
                format_money_value(income_line.value),
            )?;
            self.fill_repeating_field(
                counter,
                RepeatingFormField::HealthInsurance,
                format_money_value(income_line.health_insurance),
            )?;
            self.fill_repeating_field(
                counter,
                RepeatingFormField::TaxBase,
                format_money_value(income_line.tax_base),
            )?;
            self.fill_repeating_field(
                counter,
                RepeatingFormField::TaxAmount,
                format_money_value(income_line.tax_amount),
            )?;
            self.fill_repeating_field(
                counter,
                RepeatingFormField::TaxPaidAbroad,
                format_money_value(income_line.tax_paid_abroad),
            )?;
            self.fill_repeating_field(
                counter,
                RepeatingFormField::TaxToPay,
                format_money_value(income_line.tax_to_pay),
            )?;
            counter += 1;
        }
        self.fill_main_field(
            FormField::HealthInsuranceTotal,
            format_money_value(total.health_insurance),
        )?;
        self.fill_main_field(FormField::TaxBaseTotal, format_money_value(total.tax_base))?;
        self.fill_main_field(
            FormField::TaxAmountTotal,
            format_money_value(total.tax_amount),
        )?;
        self.fill_main_field(
            FormField::TaxPairAbroadTotal,
            format_money_value(total.tax_paid_abroad),
        )?;
        self.fill_main_field(
            FormField::TaxToPayTotal,
            format_money_value(total.tax_to_pay),
        )
    }

    pub fn to_dict(&mut self) -> Result<HashMap<String, String>> {
        self.fill_income_lines()?;
        Ok(self
            .fields
            .iter()
            .map(|(k, v)| match self.pdf_form.get_name(k.clone()) {
                Some(name) => (name, v.clone()),
                None => {
                    // pdf_forms has a bug when loading names with non ascii characters
                    // this patches one such occurence in the document
                    if k.clone() == FormField::CompanyCountry as usize {
                        ("8 Dr&#382;ava".to_string(), v.clone())
                    } else {
                        ("".to_string(), "".to_string())
                    }
                }
            })
            .filter(|(k, _)| !k.is_empty())
            .collect())
    }

    pub fn get_number_field_value(&self, field: FormField) -> Result<Decimal> {
        Decimal::from_str_radix(self.get_text_field_value(field)?.as_str(), 10)
            .map_err(|err| Error::UnexpectedCondition(err.to_string()))
    }

    pub fn get_text_field_value(&self, field: FormField) -> Result<String> {
        println!(
            "Loading text field {} of {}",
            field as usize,
            self.pdf_form.len()
        );
        match self.pdf_form.get_state(field as usize) {
            pdf_forms::FieldState::Text {
                text,
                readonly,
                required,
            } => {
                println!("Loaded text: {}", text);
                Ok(text)
            }
            _ => Err(Error::UnexpectedCondition(
                "Unsupported field type!".to_string(),
            )),
        }
    }
}

pub fn load_ams_form(input_file: String) -> Result<AmsForm> {
    match Form::load(input_file) {
        Ok(file) => Ok(AmsForm {
            pdf_form: file,
            fields: HashMap::new(),
            income_lines: Vec::new(),
        }),
        Err(err) => Err(err.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_money_value_test() {
        let pairs = [
            (dec!(3.3333), "3.33"),
            (dec!(6.6663), "6.67"),
            (dec!(1.12345), "1.12"),
            (dec!(0), "0.00"),
            (dec!(10.00), "10.00"),
            (dec!(12345.6789), "12345.68"),
        ];

        for (value, expected) in pairs {
            assert_eq!(expected, format_money_value(value))
        }
    }
}
