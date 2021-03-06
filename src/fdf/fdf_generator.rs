use std::{collections::HashMap, fs::File, io::Write};

use crate::error::Result;

static HEADER: &str = concat!("%FDF-1.2\n", "1 0 obj<</FDF<< /Fields[\n");
static FOOTER: &str = concat!(
    "] >> >>\n",
    "endobj\n",
    "trailer\n",
    "<</Root 1 0 R>>\n",
    "%%EOF"
);
static XFDF_HEADER: &str = concat!(
    "<?xml version=\"1.0\" encoding=\"UTF-8\" ?>\n",
    "<xfdf xmlns=\"http://ns.adobe.com/xfdf/\" xml:space=\"preserve\">\n",
    "  <fields>\n"
);
static XFDF_FOOTER: &str = concat!("  </fields>\n", "</xfdf>");

pub struct FdfData {
    entries: Vec<FdfDataEntry>,
}

pub struct FdfDataEntry {
    title: String,
    value: String,
}

impl FdfData {
    pub fn from_dict(dict: HashMap<String, String>) -> Self {
        Self {
            entries: dict
                .iter()
                .map(|(k, v)| FdfDataEntry {
                    title: k.clone(),
                    value: v.clone(),
                })
                .collect(),
        }
    }

    pub fn add_entry(&mut self, title: String, value: String) {
        let entry = FdfDataEntry { title, value };
        self.entries.push(entry);
    }
}

pub fn write_xfdf(data: FdfData, output_file: String) -> Result<()> {
    let mut fdf_file = File::create(output_file)?;
    fdf_file.write_all(XFDF_HEADER.as_bytes())?;
    for entry in data.entries {
        fdf_file.write_all(
            format!(
                "    <field name=\"{}\"><value>{}</value></field>\n",
                entry.title, entry.value
            )
            .as_bytes(),
        )?;
    }
    fdf_file.write_all(XFDF_FOOTER.as_bytes())?;
    Ok(())
}

pub fn write_fdf(data: FdfData, output_file: String) -> Result<()> {
    let mut fdf_file = File::create(output_file)?;
    fdf_file.write_all(HEADER.as_bytes())?;
    for entry in data.entries {
        fdf_file.write_all(format!("<</T({})/V({})>>\n", entry.title, entry.value).as_bytes())?;
    }
    fdf_file.write_all(FOOTER.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{env::temp_dir, io::Read};

    use super::*;

    #[test]
    fn add_entry_test() {
        let mut data = FdfData {
            entries: Vec::new(),
        };

        data.add_entry("test".to_string(), "value".to_string());
        data.add_entry("entry_two".to_string(), "value_two".to_string());

        let first_entry = &data.entries[0];
        let second_entry = &data.entries[1];

        assert_eq!("test".to_string(), first_entry.title);
        assert_eq!("value".to_string(), first_entry.value);
        assert_eq!("entry_two".to_string(), second_entry.title);
        assert_eq!("value_two".to_string(), second_entry.value);
    }

    #[test]
    fn from_dict_test() {
        let mut dict = HashMap::<String, String>::new();
        dict.insert("test".to_string(), "value".to_string());
        dict.insert("entry_two".to_string(), "value_two".to_string());
        let data = FdfData::from_dict(dict);

        let first_entry = &data.entries.iter().find(|x| x.title == "test").unwrap();
        let second_entry = &data
            .entries
            .iter()
            .find(|x| x.title == "entry_two")
            .unwrap();

        assert_eq!("value".to_string(), first_entry.value);
        assert_eq!("value_two".to_string(), second_entry.value);
    }

    #[test]
    fn write_fdf_test() {
        let mut dict = HashMap::<String, String>::new();
        dict.insert("test".to_string(), "value".to_string());
        dict.insert("entry_two".to_string(), "value_two".to_string());
        let mut data = FdfData::from_dict(dict);
        // entries must be sorted for predictable results
        data.entries.sort_by(|l, r| l.title.cmp(&r.title));

        let mut output_file = temp_dir();
        output_file.push("fbihtax-test.fdf");

        write_fdf(data, output_file.to_str().unwrap().to_string()).unwrap();

        let file = File::open(output_file).unwrap();
        let file_data: Vec<u8> = file.bytes().map(|x| x.unwrap()).collect();

        assert_eq!(
            concat!(
                "%FDF-1.2\n",
                "1 0 obj<</FDF<< /Fields[\n",
                "<</T(entry_two)/V(value_two)>>\n",
                "<</T(test)/V(value)>>\n",
                "] >> >>\n",
                "endobj\n",
                "trailer\n",
                "<</Root 1 0 R>>\n",
                "%%EOF"
            )
            .to_string(),
            String::from_utf8(file_data).unwrap()
        );
    }
}
