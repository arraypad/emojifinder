use failure::{Backtrace, Context, Fail};
use std::fmt;

#[derive(Debug, Fail)]
pub struct Error {
	ctx: Context<ErrorKind>,
}

impl Error {
	pub fn parse<T: AsRef<str>>(msg: T) -> Error {
		Error::from(ErrorKind::Parse(msg.as_ref().to_string()))
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.ctx.fmt(f)
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorKind {
	Parse(String),
}

impl fmt::Display for ErrorKind {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			ErrorKind::Parse(ref msg) => {
				write!(f, "Parse error: {}", msg)
			},
		}
	}
}

impl From<ErrorKind> for Error {
	fn from(kind: ErrorKind) -> Error {
		Error::from(Context::new(kind))
	}
}

impl From<Context<ErrorKind>> for Error {
	fn from(ctx: Context<ErrorKind>) -> Error {
		Error { ctx }
	}
}
