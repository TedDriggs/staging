use std::{fmt, num::ParseIntError, str::FromStr};

use staging::Staging;

#[derive(Debug)]
enum ParseError {
    TooFewParts,
    InvalidAge(ParseIntError),
}

impl From<ParseIntError> for ParseError {
    fn from(err: ParseIntError) -> Self {
        ParseError::InvalidAge(err)
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::TooFewParts => write!(f, "not enough parts in input"),
            ParseError::InvalidAge(err) => write!(f, "invalid age: {}", err),
        }
    }
}

#[derive(Debug)]
enum Error {
    Parse(ParseError),
    InvalidName,
    NameAgeMismatch,
    AgeTooHigh,
    Multiple(Vec<Error>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Parse(err) => write!(f, "parse error: {}", err),
            Error::InvalidName => write!(f, "invalid name"),
            Error::NameAgeMismatch => write!(f, "name and age do not match each other"),
            Error::AgeTooHigh => write!(f, "age too high"),
            Error::Multiple(errors) => {
                for (i, err) in errors.iter().enumerate() {
                    writeln!(f, "{}: {}", i + 1, err)?;
                }
                Ok(())
            }
        }
    }
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Self {
        Error::Parse(err)
    }
}

impl FromIterator<Error> for Error {
    fn from_iter<T: IntoIterator<Item = Error>>(iter: T) -> Self {
        let errors: Vec<Error> = iter.into_iter().collect();
        if errors.len() == 1 {
            errors.into_iter().next().unwrap()
        } else {
            Error::Multiple(errors)
        }
    }
}

#[derive(Debug, Staging)]
#[staging(error = Error, additional_errors, derive(Debug))]
struct Args {
    name: String,
    age: u32,
}

impl fmt::Display for Args {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "`{}` is {} years old", self.name, self.age)
    }
}

/// Parse a comma-separated string that starts with the name and age.
/// This returns an error if parsing is impossible, but will return `Ok` for semantic
/// errors (like invalid name or age too high).
impl FromStr for ArgsStaging {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() < 2 {
            return Err(ParseError::TooFewParts);
        }

        let name = match parts[0] {
            "" => Err(Error::InvalidName),
            n if n.chars().next().map(|c| c.is_lowercase()).unwrap() => Err(Error::InvalidName),
            n => Ok(n.to_string()),
        };

        let age = parts[1]
            .trim()
            .parse::<u32>()
            .map_err(ParseError::from)
            .map_err(Error::from)
            .and_then(|a| {
                if a > 150 {
                    Err(Error::AgeTooHigh)
                } else {
                    Ok(a)
                }
            });

        let mut additional_errors = vec![];

        if let Ok(n) = &name
            && let Ok(a) = age
        {
            if n == "Mildred" && a < 80 {
                additional_errors.push(Error::NameAgeMismatch);
            }
        }

        Ok(ArgsStaging {
            name,
            age,
            additional_errors,
        })
    }
}

/// Parse a comma-separated string that starts with the name and age.
/// This returns an error if parsing is impossible, or will return all semantic errors
/// if any are found.
impl FromStr for Args {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let staging = ArgsStaging::from_str(s)?;
        Self::try_from(staging)
    }
}

fn main() {
    let lines = [
        ",30",
        "Alice,25",
        "Bob,200",
        "bob,200",
        "charlie,thirty",
        "Mildred,70",
        "Mildred,85",
    ];

    for line in &lines {
        match line.parse::<Args>() {
            Ok(args) => println!("Parsed args: {}", args),
            Err(err) => println!("Failed to parse '{}': {:?}", line, err),
        }
    }
}
