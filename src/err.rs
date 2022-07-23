use std::fmt::Display;
use std::sync::TryLockError;

#[derive(Debug)]
pub struct ErrAlreadyRegistered;
impl Display for ErrAlreadyRegistered {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "an element with that name was already registered",)
    }
}

impl std::error::Error for ErrAlreadyRegistered {}

#[derive(Debug)]
pub struct ErrCategoryAlreadyRegistered;

impl Display for ErrCategoryAlreadyRegistered {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "a category with that name was already registered")
    }
}

impl std::error::Error for ErrCategoryAlreadyRegistered {}

#[derive(Debug)]
pub enum NSIDParseError {
    InvalidNamespace(InvalidNamespace),
    InvalidPath(InvalidPath),
    NoSeparator,
    InternerError(String),
}

impl Display for NSIDParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NSIDParseError::InvalidNamespace(ns) => Display::fmt(&ns, f),
            NSIDParseError::InvalidPath(path) => Display::fmt(&path, f),
            NSIDParseError::NoSeparator => {
                write!(f, "there was no ':' to separate the namespace and path")
            }
            NSIDParseError::InternerError(err) => {
                write!(f, "an error happened with the interner: {}", err)
            }
        }
    }
}

impl std::error::Error for NSIDParseError {}

#[derive(Debug)]
pub enum InvalidNamespace {
    Empty,
    /// Char index of the invalid character
    BadChar(usize, char),
}

#[derive(Debug)]
pub enum InvalidPath {
    Empty,
    /// Char index of the invalid character
    BadChar(usize, char),
}

impl Display for InvalidNamespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvalidNamespace::Empty => writeln!(f, "cannot have an empty namespace"),
            InvalidNamespace::BadChar(idx, c) => writeln!(f, "invalid namespace char {:?} at idx {} (valid chars are a-z, 0-9, underscore, dash)", c, idx),
        }
    }
}
impl Display for InvalidPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvalidPath::Empty => writeln!(f, "cannot have an empty path"),
            InvalidPath::BadChar(idx, c) => writeln!(f, "invalid path char {:?} at idx {} (valid chars are a-z, 0-9, underscore, dash, period, slash)", c, idx),
        }
    }
}

impl std::error::Error for InvalidNamespace {}
impl std::error::Error for InvalidPath {}

impl From<InvalidPath> for NSIDParseError {
    fn from(v: InvalidPath) -> Self {
        Self::InvalidPath(v)
    }
}

impl From<InvalidNamespace> for NSIDParseError {
    fn from(v: InvalidNamespace) -> Self {
        Self::InvalidNamespace(v)
    }
}

impl<T> From<TryLockError<T>> for NSIDParseError {
    fn from(err: TryLockError<T>) -> Self {
        let msg = err.to_string();
        Self::InternerError(msg)
    }
}
