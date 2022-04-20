use rust_decimal::Decimal;
use rust_decimal_macros::dec;

pub fn income_after_deduction(income: Decimal, deduction_percentage: Decimal) -> Decimal {
    let income_dec: Decimal = income;
    let deduction_factor: Decimal = dec!(1) - (deduction_percentage * dec!(0.01));
    income_dec * deduction_factor
}

pub fn health_insurance_part(deduced_income: Decimal) -> Decimal {
    deduced_income * dec!(0.04)
}

pub fn tax_base(deduced_income: Decimal) -> Decimal {
    deduced_income - health_insurance_part(deduced_income)
}

pub fn tax_amount(deduced_income: Decimal) -> Decimal {
    tax_base(deduced_income) * dec!(0.10)
}

pub fn health_insurance_federation(deduced_income: Decimal) -> Decimal {
    health_insurance_part(deduced_income) * dec!(0.1020)
}

pub fn health_insurance_canton(deduced_income: Decimal) -> Decimal {
    health_insurance_part(deduced_income) - health_insurance_federation(deduced_income)
}
