extern crate pdf_forms;
use std::ops::Add;

use pdf_forms::Form;

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
    value: i64,
    health_insurance: i64,
    tax_base: i64,
    tax_amount: i64,
    tax_paid_abroad: i64,
    tax_to_pay: i64,
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
    income_lines: Vec<IncomeLine>,
}

fn format_money_value(value: i64) -> String {
    format!("{},{:02}", value / 100, value % 100)
}

fn fill_field(pdf_form: &mut Form, field_index: usize, value: String) {
    match pdf_form.set_text(field_index, value) {
        Ok(_) => (),
        Err(why) => panic!("{:?}", why),
    }
}

fn fill_repeating_field(pdf_form: &mut Form, line: u32, field: RepeatingFormField, value: String) {
    fill_field(
        pdf_form,
        (field as u32 + REPEATING_FIELDS_START + line * REPEATED_FIELDS_COUNT)
            .try_into()
            .unwrap(),
        value,
    );
}

impl AmsForm {
    pub fn fill_main_field(&mut self, field: FormField, value: String) {
        fill_field(&mut self.pdf_form, field as usize, value);
    }

    pub fn add_income(&mut self, base_value: i64, tax_paid_abroad: i64) {
        let health_insurance = (base_value * 4) / 100;
        let tax_base = base_value - health_insurance;
        let tax_amount = tax_base / 10;
        let tax_to_pay = tax_amount - tax_paid_abroad;
        self.income_lines.push(IncomeLine {
            value: base_value,
            health_insurance,
            tax_base,
            tax_amount,
            tax_paid_abroad,
            tax_to_pay,
        });
    }

    pub fn save(&mut self, output_file: &str) {
        // TODO: Handle multiple pages
        self.fill_main_field(FormField::PageNumber, "1".to_string());
        self.fill_main_field(FormField::PageCount, "1".to_string());
        let total = self
            .income_lines
            .iter()
            .copied()
            .reduce(|acc, x| acc + x)
            .unwrap();
        let mut counter = 0;
        for income_line in &self.income_lines {
            fill_repeating_field(
                &mut self.pdf_form,
                counter,
                RepeatingFormField::IncomeValue,
                format_money_value(income_line.value),
            );
            fill_repeating_field(
                &mut self.pdf_form,
                counter,
                RepeatingFormField::HealthInsurance,
                format_money_value(income_line.health_insurance),
            );
            fill_repeating_field(
                &mut self.pdf_form,
                counter,
                RepeatingFormField::TaxBase,
                format_money_value(income_line.tax_base),
            );
            fill_repeating_field(
                &mut self.pdf_form,
                counter,
                RepeatingFormField::TaxAmount,
                format_money_value(income_line.tax_amount),
            );
            fill_repeating_field(
                &mut self.pdf_form,
                counter,
                RepeatingFormField::TaxPaidAbroad,
                format_money_value(income_line.tax_paid_abroad),
            );
            fill_repeating_field(
                &mut self.pdf_form,
                counter,
                RepeatingFormField::TaxToPay,
                format_money_value(income_line.tax_to_pay),
            );
            counter += 1;
        }
        self.fill_main_field(
            FormField::HealthInsuranceTotal,
            format_money_value(total.health_insurance),
        );
        self.fill_main_field(FormField::TaxBaseTotal, format_money_value(total.tax_base));
        self.fill_main_field(
            FormField::TaxAmountTotal,
            format_money_value(total.tax_amount),
        );
        self.fill_main_field(
            FormField::TaxPairAbroadTotal,
            format_money_value(total.tax_paid_abroad),
        );
        self.fill_main_field(
            FormField::TaxToPayTotal,
            format_money_value(total.tax_to_pay),
        );
        match self.pdf_form.save(output_file) {
            Ok(_) => (),
            Err(why) => panic!("{:?}", why),
        }
    }
}

pub fn load_ams_form(input_file: String) -> AmsForm {
    AmsForm {
        pdf_form: Form::load(input_file).unwrap(),
        income_lines: Vec::new(),
    }
}
