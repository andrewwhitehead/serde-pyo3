use std;
use std::fmt::{self, Display};

use serde::{de, ser};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    Message(String),
    PyErr(String),
    ExpectedBoolean,
    ExpectedBytes,
    ExpectedChar,
    ExpectedDict,
    ExpectedDictValue,
    ExpectedEnumKey,
    ExpectedEnumValue,
    ExpectedFloat,
    ExpectedInteger,
    ExpectedList,
    ExpectedListElement,
    ExpectedNone,
    ExpectedString,
    Unsupported,
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            Error::Message(msg) => msg,
            Error::PyErr(msg) => msg,
            Error::ExpectedBoolean => "expected: boolean",
            Error::ExpectedBytes => "expected: bytes",
            Error::ExpectedChar => "expected: single character",
            Error::ExpectedDict => "expected: dict",
            Error::ExpectedDictValue => "expected: dict value",
            Error::ExpectedEnumKey => "expected: non-empty dict",
            Error::ExpectedEnumValue => "expected: non-empty dict value",
            Error::ExpectedFloat => "expected: float",
            Error::ExpectedInteger => "expected: integer",
            Error::ExpectedList => "expected: list",
            Error::ExpectedListElement => "expected: list element",
            Error::ExpectedNone => "expected: none",
            Error::ExpectedString => "expected: string",
            Error::Unsupported => "unsupported input value",
        };
        formatter.write_str(msg)
    }
}

impl std::error::Error for Error {}

impl From<pyo3::PyErr> for Error {
    fn from(py_err: pyo3::PyErr) -> Error {
        Error::PyErr(format!("{:?}", py_err))
    }
}
