pub mod printer;
pub mod utils;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum OutputFormat {
    Pdf,
    Fdf,
    Xfdf,
    Json,
    Stdout,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match &self {
            OutputFormat::Pdf => "pdf",
            OutputFormat::Fdf => "fdf",
            OutputFormat::Xfdf => "xfdf",
            OutputFormat::Json => "json",
            OutputFormat::Stdout => "stdout",
        })
    }
}

impl std::fmt::Debug for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match &self {
            OutputFormat::Pdf => "pdf",
            OutputFormat::Fdf => "fdf",
            OutputFormat::Xfdf => "xfdf",
            OutputFormat::Json => "json",
            OutputFormat::Stdout => "stdout",
        })
    }
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pdf" => Ok(OutputFormat::Pdf),
            "fdf" => Ok(OutputFormat::Fdf),
            "xfdf" => Ok(OutputFormat::Xfdf),
            "json" => Ok(OutputFormat::Json),
            "stdout" => Ok(OutputFormat::Stdout),
            _ => Err("Unknown format passed!".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn output_format_name_normal_test() {
        assert_eq!(OutputFormat::Pdf, OutputFormat::from_str("pdf").unwrap());
        assert_eq!(OutputFormat::Json, OutputFormat::from_str("json").unwrap());
        assert_eq!(OutputFormat::Fdf, OutputFormat::from_str("fdf").unwrap());
        assert_eq!(OutputFormat::Xfdf, OutputFormat::from_str("xfdf").unwrap());
        assert_eq!(
            OutputFormat::Stdout,
            OutputFormat::from_str("stdout").unwrap()
        );
    }

    #[test]
    fn output_format_name_unsupported_test() {
        assert!(OutputFormat::from_str("yml").is_err());
        assert!(OutputFormat::from_str("xml").is_err());
    }

    #[test]
    fn output_format_name_wrong_case_test() {
        assert!(OutputFormat::from_str("PDF").is_err());
        assert!(OutputFormat::from_str("JSON").is_err());
        assert!(OutputFormat::from_str("FDF").is_err());
        assert!(OutputFormat::from_str("XFDF").is_err());
        assert!(OutputFormat::from_str("STDOUT").is_err());
    }

    #[test]
    fn output_format_display_test() {
        assert_eq!("format is pdf", format!("format is {}", OutputFormat::Pdf));
        assert_eq!(
            "format is json",
            format!("format is {}", OutputFormat::Json)
        );
        assert_eq!("format is fdf", format!("format is {}", OutputFormat::Fdf));
        assert_eq!(
            "format is xfdf",
            format!("format is {}", OutputFormat::Xfdf)
        );
    }
}
