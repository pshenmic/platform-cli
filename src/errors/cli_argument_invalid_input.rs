use std::fmt;

#[derive(Debug)]
pub struct CommandLineArgumentInvalidInput(String);

impl fmt::Display for CommandLineArgumentInvalidInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Input data is not valid: {}", &self.0)
    }
}

impl From<&str> for CommandLineArgumentInvalidInput {
    fn from(value: &str) -> Self {
        CommandLineArgumentInvalidInput(String::from(value))
    }
}