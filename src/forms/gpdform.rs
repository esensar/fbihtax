extern crate pdf_forms;
extern crate rust_decimal;
use std::{collections::HashMap, ops::Add};

use pdf_forms::Form;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::{config::UserConfig, error::FbihtaxResult};

use super::formutils::{fill_field, format_money_value};

#[derive(Clone, Copy)]
pub enum FormField {
    UserJmbg = 0,
    UserName = 1,
    TaxYearLast2 = 2,
    UserAddress = 3,
    UserPhone = 4,
    UserEmail = 5,
    GipIncome = 6,
    SprExpense = 7,
    SprIncome = 8,
    AgroExpense = 9,
    AgroIncome = 10,
    RentExpense = 11,
    RentIncome = 12,
    Rent2Expense = 13,
    Rent2Income = 14,
    AugExpense = 15,
    AugIncome = 16,
    PreviousExpense = 17,
    ExpenseSum = 18,
    IncomeSum = 19,
    ExpenseTotal = 20,
    IncomeTotal = 21,
    PersonalDeduction = 22,
    HealthDeduction = 23,
    InterestDeduction = 24,
    TotalDeduction = 25,
    UserNameP2 = 26,
    UserJmbgP2 = 27,
    TaxYearLast2P2 = 28,
    ExpenseTotalP2 = 29,
    IncomeTotalP2 = 30,
    TotalDeductionP2 = 31,
    TaxBaseP2 = 32,
    TaxTotalP2 = 33,
    TaxDeductionF27P2 = 34,
    F29P2 = 35,
    TaxPaidP2 = 36,
    ExternalPaidTaxP2 = 37,
    RequestReturnP2 = 38,
    ReturnTotalP2 = 39,
    AccountNumberP2 = 40,
    DateP2 = 41,
    PeriodStart = 42,
    PeriodEnd = 43,
}

pub struct GpdForm {
    pdf_form: Form,
    fields: HashMap<usize, String>,
    gip_info: Option<TaxInfo>,
    ams_info: Option<TaxInfo>,
    deductions: Deductions,
}

struct TaxInfo {
    income: Decimal,
    tax_paid: Decimal,
}

struct Deductions {
    personal: Decimal,
    health: Decimal,
    interest: Decimal,
}

impl Add for TaxInfo {
    type Output = TaxInfo;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            income: self.income + rhs.income,
            tax_paid: self.tax_paid + rhs.tax_paid,
        }
    }
}

impl Clone for TaxInfo {
    fn clone(&self) -> Self {
        Self {
            income: self.income,
            tax_paid: self.tax_paid,
        }
    }
}

impl Default for Deductions {
    fn default() -> Self {
        Self {
            personal: dec!(0),
            health: dec!(0),
            interest: dec!(0),
        }
    }
}

impl Deductions {
    fn get_total(&self) -> Decimal {
        return self.personal + self.health + self.interest;
    }
}

impl GpdForm {
    pub fn fill_field(&mut self, field: FormField, value: String) -> FbihtaxResult<()> {
        self.fields.insert(field as usize, value.clone());
        fill_field(&mut self.pdf_form, field as usize, value)
    }

    pub fn fill_user_info(&mut self, value: &UserConfig) -> FbihtaxResult<()> {
        self.fill_field(FormField::UserName, value.name.clone())?;
        self.fill_field(FormField::UserNameP2, value.name.clone())?;
        self.fill_field(FormField::UserJmbg, value.jmbg.clone())?;
        self.fill_field(FormField::UserJmbgP2, value.jmbg.clone())?;
        self.fill_field(FormField::UserAddress, value.address.clone())?;
        self.fill_field(
            FormField::UserPhone,
            value.phone.clone().unwrap_or("".to_string()),
        )?;
        self.fill_field(
            FormField::UserEmail,
            value.email.clone().unwrap_or("".to_string()),
        )
    }

    pub fn fill_year_info(&mut self, year: String) -> FbihtaxResult<()> {
        let year_last_2 = &year[2..year.len()];
        self.fill_field(FormField::PeriodStart, "0101".to_string())?;
        self.fill_field(FormField::PeriodEnd, "3112".to_string())?;
        self.fill_field(FormField::TaxYearLast2, year_last_2.to_string())?;
        self.fill_field(FormField::TaxYearLast2P2, year_last_2.to_string())
    }

    pub fn add_gip_info(&mut self, gip_income: Decimal, gip_tax_paid: Decimal) {
        self.gip_info = Some(TaxInfo {
            income: gip_income,
            tax_paid: gip_tax_paid,
        });
    }

    pub fn add_ams_info(&mut self, ams_income: Decimal, ams_tax_paid: Decimal) {
        self.ams_info = Some(TaxInfo {
            income: ams_income,
            tax_paid: ams_tax_paid,
        });
    }

    pub fn add_deductions(&mut self, personal: Decimal, health: Decimal, interest: Decimal) {
        self.deductions = Deductions {
            personal,
            health,
            interest,
        };
    }

    pub fn to_dict(&mut self) -> FbihtaxResult<HashMap<String, String>> {
        let mut total_tax_info = TaxInfo {
            income: dec!(0),
            tax_paid: dec!(0),
        };
        match &self.gip_info.clone() {
            Some(gip) => {
                self.fill_field(FormField::GipIncome, format_money_value(gip.income))?;
                total_tax_info = total_tax_info + gip.clone();
            }
            None => {}
        }
        match &self.ams_info.clone() {
            Some(ams) => {
                self.fill_field(FormField::AugIncome, format_money_value(ams.income))?;
                total_tax_info = total_tax_info + ams.clone();
            }
            None => {}
        }
        self.fill_field(
            FormField::IncomeSum,
            format_money_value(total_tax_info.income),
        )?;
        // TODO: Add support for expenses?
        self.fill_field(FormField::ExpenseSum, format_money_value(dec!(0)))?;
        self.fill_field(FormField::ExpenseTotal, format_money_value(dec!(0)))?;
        self.fill_field(FormField::ExpenseTotalP2, format_money_value(dec!(0)))?;
        self.fill_field(
            FormField::IncomeTotal,
            format_money_value(total_tax_info.income),
        )?;
        self.fill_field(
            FormField::IncomeTotalP2,
            format_money_value(total_tax_info.income),
        )?;
        self.fill_field(
            FormField::PersonalDeduction,
            format_money_value(self.deductions.personal),
        )?;
        self.fill_field(
            FormField::HealthDeduction,
            format_money_value(self.deductions.health),
        )?;
        self.fill_field(
            FormField::InterestDeduction,
            format_money_value(self.deductions.interest),
        )?;
        self.fill_field(
            FormField::TotalDeduction,
            format_money_value(self.deductions.get_total()),
        )?;
        self.fill_field(
            FormField::TotalDeductionP2,
            format_money_value(self.deductions.get_total()),
        )?;
        let tax_base = total_tax_info.income - dec!(0) - self.deductions.get_total();
        let tax_to_pay = tax_base * dec!(0.1);
        self.fill_field(FormField::TaxBaseP2, format_money_value(tax_base))?;
        self.fill_field(FormField::TaxTotalP2, format_money_value(tax_to_pay))?;
        self.fill_field(
            FormField::TaxPaidP2,
            format_money_value(total_tax_info.tax_paid),
        )?;
        self.fill_field(
            FormField::ReturnTotalP2,
            format_money_value(tax_to_pay - total_tax_info.tax_paid),
        )?;
        Ok(self
            .fields
            .iter()
            .map(|(k, v)| match self.pdf_form.get_name(k.clone()) {
                Some(name) => (name, v.clone()),
                None => ("".to_string(), "".to_string()),
            })
            .filter(|(k, _)| !k.is_empty())
            .collect())
    }
}

pub fn load_gpd_form(input_file: String) -> FbihtaxResult<GpdForm> {
    match Form::load(input_file) {
        Ok(file) => Ok(GpdForm {
            pdf_form: file,
            fields: HashMap::new(),
            gip_info: None,
            ams_info: None,
            deductions: Deductions::default(),
        }),
        Err(err) => Err(err.into()),
    }
}
