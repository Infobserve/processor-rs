use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigurationError {
    #[error("Unrecognized value for `workers` key: {0}")]
    BadWorkersKeyValue(String),
    #[error("No yara rules could be loaded")]
    NoYaraRulesError,
    #[error("Number of workers cannot be negative")]
    NegativeWorkersError
}

#[derive(Error, Debug)]
pub enum DeserializationError {
    #[error("Empty '{0}' value when deserializing event")]
    NoValueError(String)
}
