use std::fmt;

#[derive(Debug)]
pub struct CommandLineArgumentMissingError(String);

impl fmt::Display for CommandLineArgumentMissingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Missing command line argument {}", &self.0)
    }
}

impl From<&str> for CommandLineArgumentMissingError {
    fn from(value: &str) -> Self {
        CommandLineArgumentMissingError(String::from(value))
    }
}