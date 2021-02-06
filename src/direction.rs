use std::convert::TryFrom;
use std::fmt;
use crate::json::{
	Json,
	AsJson
};

/// Internationalized string direction.
///
/// Specifies the direction used to read a string.
/// This can be either left-to-right (`"ltr"`) or right-to-left (`"rtl"`).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Direction {
	/// Left-to-right direction.
	Ltr,

	/// Right-to-left direction.
	Rtl
}

impl<'a> TryFrom<&'a str> for Direction {
	type Error = &'a str;

	/// Convert the strings `"rtl"` and `"ltr"` into a `Direction`.
	fn try_from(name: &'a str) -> Result<Direction, &'a str> {
		match name {
			"ltr" => Ok(Direction::Ltr),
			"rtl" => Ok(Direction::Rtl),
			_ => Err(name)
		}
	}
}

impl fmt::Display for Direction {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Direction::Ltr => write!(f, "ltr"),
			Direction::Rtl => write!(f, "rtl")
		}
	}
}

impl<J: Json> AsJson<J> for Direction {
	/// Convert the direction into a JSON string.
	/// Either `"rtl"` or `"ltr"`.
	fn as_json(&self) -> J {
		match self {
			Direction::Ltr => "ltr".as_json(),
			Direction::Rtl => "rtl".as_json()
		}
	}
}
