use std::collections::HashMap;

pub fn fill_template(template: String, data: HashMap<String, String>) -> String {
    let mut result = template.clone();
    for (key, value) in data {
        result = result.replace(format!("{{{}}}", key).as_str(), value.as_str());
    }
    return result;
}
