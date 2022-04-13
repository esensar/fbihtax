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
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

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
