mod amsform;
use amsform::FormField;

fn main() {
    let mut form = amsform::load_ams_form("tax.pdf");

    form.fill_main_field(FormField::UserName, "Ensar".to_string());
    form.add_income(100000, 0);

    form.save("taxresult.pdf");
}
