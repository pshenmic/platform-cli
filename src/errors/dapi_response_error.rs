use std::fmt;

#[derive(Debug)]
pub struct DapiResponseError(String);

impl From<&str> for DapiResponseError {
    fn from(value: &str) -> Self {
        DapiResponseError(String::from(value))
    }
}

impl fmt::Display for DapiResponseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}
