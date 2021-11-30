extern crate pdf_forms;
use pdf_forms::Form;

use std::collections::HashMap;

fn main() {
    let form_fields = HashMap::from([
        ("page_number", 0),
        ("page_count", 1),
        ("user_name", 2),
        ("user_jmbg", 3),
        ("user_address", 4),
        ("payment_date_day", 5),
        ("payment_date_month", 6),
        ("payment_date_year", 7),
        ("tax_period_month", 8),
        ("tax_period_year_last2", 9),
        ("company_name", 10),
        ("company_address", 11),
        ("company_country", 12),
        ("health_insurance_total", 43),
        ("tax_base_total", 44),
        ("tax_amount_total", 45),
        ("tax_paid_abroad_total", 46),
        ("tax_to_pay_total", 47),
        ("date", 48),
    ]);

    let repeating_fields = HashMap::from([
        ("income_value", 0),
        ("health_insurance", 1),
        ("tax_base", 2),
        ("tax_amount", 3),
        ("tax_paid_abroad", 4),
        ("tax_to_pay", 5),
    ]);
    let repeating_fields_start = 13;
    let repeated_lines = 5;

    let mut form = Form::load("tax.pdf").unwrap();

    for (key, index) in form_fields {
        match form.set_text(index, format!("{}", key)) {
            Ok(res) => println!("Field {} set! Result: {:?}", index, res),
            Err(why) => panic!("{:?}", why),
        }
    }
    for i in 0..repeated_lines {
        repeating_fields.iter().for_each(|(key, index)| {
            match form.set_text(
                repeating_fields_start + index + i * repeating_fields.len(),
                format!("{} line {}", key, i),
            ) {
                Ok(res) => println!("Field {} set! Result: {:?}", index, res),
                Err(why) => panic!("{:?}", why),
            }
        })
    }
    let result = form.save("taxresult.pdf");
    match result {
        Ok(res) => println!("saving success: {:?}", res),
        Err(why) => panic!("{:?}", why),
    }
}
