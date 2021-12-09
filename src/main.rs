mod amsform;
mod config;
use std::path::Path;

use amsform::FormField;

fn main() {
    let config = config::parse_config(".fbihtax.json");
    let mut form = amsform::load_ams_form(config.pdf.cache_location);

    form.fill_main_field(FormField::UserName, "Ensar".to_string());
    form.add_income(100000, 0);

    form.save(
        Path::new(config.output_location.as_str())
            .join("amsform.pdf")
            .to_str()
            .expect("Output location seems to be invalid!"),
    );
}
