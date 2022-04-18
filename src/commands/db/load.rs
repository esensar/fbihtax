extern crate clap;

use clap::Parser;

use crate::{
    config::Config,
    db::{self, TaxDb},
    forms::amsform::{self, FormField},
};

#[derive(Parser, Debug)]
pub struct LoadArgs {
    #[clap(short, long)]
    file: String,
}

pub fn handle_command(config: Config, args: &LoadArgs) {
    let form = amsform::load_ams_form(args.file.clone());

    let total_paid = form.get_number_field_value(FormField::TaxToPayTotal);
    let income = form.get_number_field_value(FormField::TaxBaseTotal)
        + form.get_number_field_value(FormField::HealthInsuranceTotal);

    let invoice_date = [
        form.get_text_field_value(FormField::PaymentDateYear),
        "-".to_string(),
        form.get_text_field_value(FormField::PaymentDateMonth),
        "-".to_string(),
        form.get_text_field_value(FormField::PaymentDateDay),
    ]
    .concat();

    let mut tax_db: TaxDb = db::parse_db_with_default(config.db_location.as_str());
    tax_db.add_ams_info(
        db::AmsInfo {
            income_total: income,
            tax_paid: total_paid,
        },
        invoice_date,
    );
    tax_db.write_to_file(config.db_location.as_str());
}
