use std::{error::Error, fmt::Display, io};

use crate::format::OutputFormat;

#[derive(Debug)]
pub enum FbihtaxError {
    Io(io::Error),
    Network(reqwest::Error),
    Json(serde_json::Error),
    Pdf(PdfErrorKind),
    UserError(UserErrorKind),
    UnexpectedCondition(String),
}

#[derive(Debug)]
pub enum UserErrorKind {
    UnsupportedOutputFormat(OutputFormat),
    MissingConfig(String, String),
    Generic(String),
}

#[derive(Debug)]
pub enum PdfErrorKind {
    Value(pdf_forms::ValueError),
    Load(pdf_forms::LoadError),
}

impl Error for FbihtaxError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            FbihtaxError::Io(err) => Some(err),
            FbihtaxError::Json(err) => Some(err),
            FbihtaxError::Pdf(err) => match err {
                PdfErrorKind::Value(inner) => Some(inner),
                PdfErrorKind::Load(inner) => Some(inner),
            },
            FbihtaxError::Network(err) => Some(err),
            _ => None,
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

impl Display for FbihtaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FbihtaxError::Io(err) => err.fmt(f),
            FbihtaxError::Json(err) => err.fmt(f),
            FbihtaxError::Pdf(err) => err.fmt(f),
            FbihtaxError::Network(err) => err.fmt(f),
            FbihtaxError::UserError(message) => message.fmt(f),
            FbihtaxError::UnexpectedCondition(message) => message.fmt(f),
        }
    }
}

impl Display for UserErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserErrorKind::UnsupportedOutputFormat(format) => {
                f.write_fmt(format_args!("Unsupported output format: {}", format))
            }
            UserErrorKind::MissingConfig(config_name, argument) => f.write_fmt(format_args!(
                "Missing {}. Try to provide it with {}",
                config_name, argument
            )),
            UserErrorKind::Generic(message) => message.fmt(f),
        }
    }
}

impl Display for PdfErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PdfErrorKind::Value(err) => err.fmt(f),
            PdfErrorKind::Load(err) => err.fmt(f),
        }
    }
}

impl From<io::Error> for FbihtaxError {
    fn from(err: io::Error) -> Self {
        FbihtaxError::Io(err)
    }
}

impl From<reqwest::Error> for FbihtaxError {
    fn from(err: reqwest::Error) -> Self {
        FbihtaxError::Network(err)
    }
}

impl From<serde_json::Error> for FbihtaxError {
    fn from(err: serde_json::Error) -> Self {
        FbihtaxError::Json(err)
    }
}

impl From<pdf_forms::ValueError> for FbihtaxError {
    fn from(err: pdf_forms::ValueError) -> Self {
        FbihtaxError::Pdf(PdfErrorKind::Value(err))
    }
}

impl From<pdf_forms::LoadError> for FbihtaxError {
    fn from(err: pdf_forms::LoadError) -> Self {
        FbihtaxError::Pdf(PdfErrorKind::Load(err))
    }
}

pub type FbihtaxResult<T> = std::result::Result<T, FbihtaxError>;
