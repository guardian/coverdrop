use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum GeneratorError {
    #[error("IO")]
    Io(#[from] io::Error),
    #[error("Invalid word list line")]
    InvalidWordListLine,
    #[error("Password format is incorrect, should be words separated by spaces")]
    PasswordFormatError,
    #[error("Password contains one or more misspelt words")]
    MisspeltWords(Vec<String>),
}
