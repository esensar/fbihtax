use std::{error, fmt::Display, io};

use crate::format::OutputFormat;

#[derive(Debug)]
pub enum Error {
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

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::Io(err) => Some(err),
            Error::Json(err) => Some(err),
            Error::Pdf(err) => match err {
                PdfErrorKind::Value(inner) => Some(inner),
                PdfErrorKind::Load(inner) => Some(inner),
            },
            Error::Network(err) => Some(err),
            _ => None,
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        self.source()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(err) => err.fmt(f),
            Error::Json(err) => err.fmt(f),
            Error::Pdf(err) => err.fmt(f),
            Error::Network(err) => err.fmt(f),
            Error::UserError(message) => message.fmt(f),
            Error::UnexpectedCondition(message) => message.fmt(f),
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

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Network(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json(err)
    }
}

impl From<pdf_forms::ValueError> for Error {
    fn from(err: pdf_forms::ValueError) -> Self {
        Error::Pdf(PdfErrorKind::Value(err))
    }
}

impl From<pdf_forms::LoadError> for Error {
    fn from(err: pdf_forms::LoadError) -> Self {
        Error::Pdf(PdfErrorKind::Load(err))
    }
}

pub type Result<T> = std::result::Result<T, Error>;
