extern crate pdf_forms;
extern crate rust_decimal;
use std::collections::HashMap;

use pdf_forms::Form;

use crate::config::UserConfig;

use super::formutils::fill_field;

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
    TaxPaidP2 = 35,
    F29P2 = 36,
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
}

impl GpdForm {
    pub fn fill_field(&mut self, field: FormField, value: String) {
        self.fields.insert(field as usize, value.clone());
        fill_field(&mut self.pdf_form, field as usize, value);
    }

    pub fn fill_user_info(&mut self, value: &UserConfig) {
        self.fill_field(FormField::UserName, value.name.clone());
        self.fill_field(FormField::UserNameP2, value.name.clone());
        self.fill_field(FormField::UserJmbg, value.jmbg.clone());
        self.fill_field(FormField::UserJmbgP2, value.jmbg.clone());
        self.fill_field(FormField::UserAddress, value.address.clone());
        self.fill_field(
            FormField::UserPhone,
            value.phone.clone().unwrap_or("".to_string()),
        );
        self.fill_field(
            FormField::UserEmail,
            value.email.clone().unwrap_or("".to_string()),
        );
    }

    pub fn fill_year_info(&mut self, year: String) {
        let year_last_2 = &year[2..year.len()];
        self.fill_field(FormField::PeriodStart, "0101".to_string());
        self.fill_field(FormField::PeriodEnd, "3112".to_string());
        self.fill_field(FormField::TaxYearLast2, year_last_2.to_string());
        self.fill_field(FormField::TaxYearLast2P2, year_last_2.to_string());
    }

    pub fn to_dict(&mut self) -> HashMap<String, String> {
        self.fields
            .iter()
            .map(|(k, v)| match self.pdf_form.get_name(k.clone()) {
                Some(name) => (name, v.clone()),
                None => ("".to_string(), "".to_string()),
            })
            .filter(|(k, _)| !k.is_empty())
            .collect()
    }
}

pub fn load_gpd_form(input_file: String) -> GpdForm {
    let form = GpdForm {
        pdf_form: Form::load(input_file).unwrap(),
        fields: HashMap::new(),
    };
    for i in 0..(form.pdf_form.len() - 1) {
        println!("Index: {}, Name: {}", i, form.pdf_form.get_name(i).unwrap());
    }
    return form;
}
