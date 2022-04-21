extern crate clap;
extern crate rust_decimal;

use clap::Parser;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::{
    config::Config,
    db::{self, TaxDb},
    error::{self, Error, UserErrorKind},
    taxcalculator,
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

pub fn handle_command(config: Config, args: &InsertArgs) -> error::Result<()> {
    let income = match &args.income {
        Some(inc) => taxcalculator::income_after_deduction(
            inc.round_dp(2),
            args.deduction_percentage.round_dp(2),
        ),
        None => match &args.deduced_income {
            Some(deduced_income) => deduced_income.clone(),
            None => {
                return Err(Error::UserError(UserErrorKind::Generic(
                    "Provide either --income or --deduced-income!".to_string(),
                )))
            }
        },
    };
    let mut tax_db: TaxDb = db::parse_db_with_default(config.db_location.as_str());
    tax_db.add_ams_info(
        db::AmsInfo {
            income_total: income,
            tax_paid: taxcalculator::tax_amount(income),
        },
        args.invoice_date.clone(),
    );
    tax_db.write_to_file(config.db_location.as_str())
}
