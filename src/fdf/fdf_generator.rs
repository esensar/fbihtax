use std::{collections::HashMap, fs::File, io::Write};

static HEADER: &str = concat!("%FDF-1.2\n", "1 0 obj<</FDF<< /Fields[\n");
static FOOTER: &str = concat!(
    "] >> >>\n",
    "endobj\n",
    "trailer\n",
    "<</Root 1 0 R>>\n",
    "%%EOF"
);

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

pub fn write_fdf(data: FdfData, output_file: String) {
    let mut fdf_file = File::create(output_file).expect("Failed creating fdf file!");
    fdf_file
        .write_all(HEADER.as_bytes())
        .expect("Writing fdf file failed!");
    for entry in data.entries {
        fdf_file
            .write_all(format!("<</T({})/V({})>>\n", entry.title, entry.value).as_bytes())
            .expect("Writing fdf file failed!");
    }
    fdf_file
        .write_all(FOOTER.as_bytes())
        .expect("Writing fdf file failed!");
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

        let first_entry = &data.entries[0];
        let second_entry = &data.entries[1];

        assert_eq!("test".to_string(), first_entry.title);
        assert_eq!("value".to_string(), first_entry.value);
        assert_eq!("entry_two".to_string(), second_entry.title);
        assert_eq!("value_two".to_string(), second_entry.value);
    }

    #[test]
    fn write_fdf_test() {
        let mut dict = HashMap::<String, String>::new();
        dict.insert("test".to_string(), "value".to_string());
        dict.insert("entry_two".to_string(), "value_two".to_string());
        let data = FdfData::from_dict(dict);

        let mut output_file = temp_dir();
        output_file.push("fbihtax-test.fdf");

        write_fdf(data, output_file.to_str().unwrap().to_string());

        let file = File::open(output_file).unwrap();
        let file_data: Vec<u8> = file.bytes().map(|x| x.unwrap()).collect();

        assert_eq!(
            concat!(
                "%FDF-1.2\n",
                "1 0 obj<</FDF<< /Fields[\n",
                "<</T(test)/V(value)>>\n",
                "<</T(entry_two)/V(value_two)>>\n",
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
