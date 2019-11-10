use std::error::Error;
use std::fmt;
use std::string::FromUtf8Error;
use xml::writer::Error as WriterError;

#[derive(Debug)]
pub enum SoapBuilderError {
    WriterError(WriterError),
    FromUtf8Error(FromUtf8Error),
}

impl From<WriterError> for SoapBuilderError {
    fn from(err: WriterError) -> Self {
        Self::WriterError(err)
    }
}

impl From<FromUtf8Error> for SoapBuilderError {
    fn from(err: FromUtf8Error) -> Self {
        Self::FromUtf8Error(err)
    }
}

impl fmt::Display for SoapBuilderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl Error for SoapBuilderError {
    fn description(&self) -> &str {
        match self {
            Self::WriterError(err) => err.description(),
            Self::FromUtf8Error(err) => err.description(),
        }
    }
}
