use pdf_forms::Form;
use rust_decimal::Decimal;

use crate::error::{Error, Result};

pub fn format_money_value(value: Decimal) -> String {
    value.round_dp(2).to_string()
}

pub fn fill_field(pdf_form: &mut Form, field_index: usize, value: String) -> Result<()> {
    pdf_form.set_text(field_index, value).map_err(Error::from)
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;

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
