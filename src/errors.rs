use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct NonPositiveWorkersError;
impl fmt::Display for NonPositiveWorkersError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Number of workers must be positive")
    }
}
impl Error for NonPositiveWorkersError {}
