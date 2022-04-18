extern crate clap;
extern crate rust_decimal;

use clap::Parser;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::{
    config::Config,
    db::{self, TaxDb},
};

#[derive(Parser, Debug)]
pub struct InsertArgs {
    #[clap(
        short,
        long,
        help = "Decimal income value in BAM (will be rounded to 2 decimals) after deduction"
    )]
    deduced_income: Option<Decimal>,
    #[clap(
        short,
        long,
        help = "Decimal income value in BAM (will be rounded to 2 decimals)"
    )]
    income: Option<Decimal>,
    #[clap(
        long,
        help = "Tax deduction percentage (20 default, 30 for income from authored work). Applied only when income is used and not deduced income",
        default_value_t = dec!(20)
    )]
    deduction_percentage: Decimal,
    #[clap(long, help = "Invoice date (YYYY-MM-DD)")]
    invoice_date: String,
}

pub fn handle_command(config: Config, args: &InsertArgs) {
    let income = match &args.income {
        Some(inc) => {
            let deduction_factor: Decimal =
                dec!(1) - (args.deduction_percentage.round_dp(2) * dec!(0.01));
            inc * deduction_factor
        }
        None => match &args.deduced_income {
            Some(deduced_income) => deduced_income.clone(),
            None => panic!("Provide either --income or --deduced_income!"),
        },
    };
    // TODO: extract tax calculations into a common module
    let health_insurance = income * dec!(0.04);
    let tax_base = income - health_insurance;
    let tax_amount: Decimal = tax_base * dec!(0.10);
    let mut tax_db: TaxDb = db::parse_db_with_default(config.db_location.as_str());
    tax_db.add_ams_info(
        db::AmsInfo {
            income_total: income,
            tax_paid: tax_amount,
        },
        args.invoice_date.clone(),
    );
    tax_db.write_to_file(config.db_location.as_str());
}
