//! When serializing or deserializing Erlang binary terms goes wrong.

use std::error;
use std::fmt::{self, Debug, Display};
use std::io;
use std::result;
use std::str::FromStr;

pub struct Error {
    err: Box<ErrorImpl>,
}

pub type Result<T> = result::Result<T, Error>;

impl Error {
    pub fn line(&self) -> usize {
        self.err.line
    }

    pub fn column(&self) -> usize {
        self.err.column
    }

    pub fn classify(&self) -> Category {
        match self.err.code {
            ErrorCode::Message(_) => Category::Data,
            ErrorCode::Io(_) => Category::Io,
            ErrorCode::ValueNotEncodable(_) => Category::Data,
        }
    }

    pub fn is_io(&self) -> bool {
        self.classify() == Category::Io
    }

    pub fn is_syntax(&self) -> bool {
        self.classify() == Category::Syntax
    }

    pub fn is_data(&self) -> bool {
        self.classify() == Category::Data
    }

    pub fn is_eof(&self) -> bool {
        self.classify() == Category::Eof
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Category {
    Io,
    Syntax,
    Data,
    Eof,
}

impl From<Error> for io::Error {
    fn from(j: Error) -> Self {
        if let ErrorCode::Io(err) = j.err.code {
            err
        } else {
            match j.classify() {
                Category::Io => unreachable!(),
                Category::Syntax | Category::Data => io::Error::new(io::ErrorKind::InvalidData, j),
                Category::Eof => io::Error::new(io::ErrorKind::UnexpectedEof, j),
            }
        }
    }
}

struct ErrorImpl {
    code: ErrorCode,
    line: usize,
    column: usize,
}

pub enum ErrorCode {
    /// Catchall for syntax error messages
    Message(Box<str>),
    Io(io::Error),
    ValueNotEncodable(Box<str>)
}

impl Error {
    pub fn syntax(code: ErrorCode, line: usize, column: usize) -> Self {
        Error {
            err: Box::new(ErrorImpl {
                code,
                line,
                column,
            }),
        }
    }

    pub fn io(error: io::Error) -> Self {
        Error {
            err: Box::new(ErrorImpl {
                code: ErrorCode::Io(error),
                line: 0,
                column: 0,
            }),
        }
    }

    pub fn data(code: ErrorCode) -> Self {
        Error {
            err: Box::new(ErrorImpl {
                code,
                line: 0,
                column: 0,
            })
        }
    }

    pub fn fix_position<F>(self, f: F) -> Self
        where
            F: FnOnce(ErrorCode) -> Error,
    {
        if self.err.line == 0 {
            f(self.err.code)
        } else {
            self
        }
    }
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorCode::Message(ref msg) => f.write_str(msg),
            ErrorCode::Io(ref err) => Display::fmt(err, f),
            ErrorCode::ValueNotEncodable(ref msg) => f.write_fmt(format_args!("Passed vallue cannot be encoded: {}", msg))
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self.err.code {
            ErrorCode::Io(ref err) => Some(err),
            _ => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&*self.err, f)
    }
}

impl Display for ErrorImpl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.line == 0 {
            Display::fmt(&self.code, f)
        } else {
            write!(
                f,
                "{} at line {} column {}",
                self.code, self.line, self.column
            )
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Error({:?}, line: {}, column: {})",
            self.err.code.to_string(),
            self.err.line,
            self.err.column
        )
    }
}

//impl de::Error for Error {
//    fn custom<T: Display>(msg: T) -> Error {
//        make_error(msg.to_string())
//    }
//
//    fn invalid_type(unexp: de::Unexpected, exp: &dyn de::Expected) -> Self {
//        if let de::Unexpected::Unit = unexp {
//            Error::custom(format_args!("invalid type: null, expected {}", exp))
//        } else {
//            Error::custom(format_args!("invalid type: {}, expected {}", unexp, exp))
//        }
//    }
//}

//impl ser::Error for Error {
//    fn custom<T: Display>(msg: T) -> Error {
//        make_error(msg.to_string())
//    }
//}

fn make_error(mut msg: String) -> Error {
    let (line, column) = parse_line_col(&mut msg).unwrap_or((0, 0));
    Error {
        err: Box::new(ErrorImpl {
            code: ErrorCode::Message(msg.into_boxed_str()),
            line: line,
            column: column,
        }),
    }
}

fn parse_line_col(msg: &mut String) -> Option<(usize, usize)> {
    let start_of_suffix = match msg.rfind(" at line ") {
        Some(index) => index,
        None => return None,
    };

    let start_of_line = start_of_suffix + " at line ".len();
    let mut end_of_line = start_of_line;
    while starts_with_digit(&msg[end_of_line..]) {
        end_of_line += 1;
    }

    if !msg[end_of_line..].starts_with(" column ") {
        return None;
    }

    let start_of_column = end_of_line + " column ".len();
    let mut end_of_column = start_of_column;
    while starts_with_digit(&msg[end_of_column..]) {
        end_of_column += 1;
    }

    if end_of_column < msg.len() {
        return None;
    }

    let line = match usize::from_str(&msg[start_of_line..end_of_line]) {
        Ok(line) => line,
        Err(_) => return None,
    };
    let column = match usize::from_str(&msg[start_of_column..end_of_column]) {
        Ok(column) => column,
        Err(_) => return None,
    };

    msg.truncate(start_of_suffix);
    Some((line, column))
}

fn starts_with_digit(slice: &str) -> bool {
    match slice.as_bytes().get(0) {
        None => false,
        Some(&byte) => byte >= b'0' && byte <= b'9',
    }
}