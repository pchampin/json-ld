use std::hash::{Hash, Hasher};
use iref::IriBuf;
use langtag::LanguageTag;
use cc_traits::MapInsert;
use crate::{
	json::{
		self,
		Json,
		AsJson
	},
	Id,
	object,
	LangString,
	Direction,
	syntax::{
		Keyword,
		Type
	}
};

/// Literal value.
#[derive(Clone)]
pub enum Literal<J: Json> {
	/// The `null` value.
	Null,

	/// Boolean value.
	Boolean(bool),

	/// Number.
	Number(J::Number),

	/// String.
	String(String)
}

impl<J: Json> PartialEq for Literal<J> {
	fn eq(&self, other: &Literal<J>) -> bool {
		use Literal::*;
		match (self, other) {
			(Null, Null) => true,
			(Boolean(a), Boolean(b)) => a == b,
			(Number(a), Number(b)) => a == b,
			(String(a), String(b)) => a == b,
			_ => false
		}
	}
}

impl<J: Json> Eq for Literal<J> {}

impl<J: Json> Hash for Literal<J> {
	fn hash<H: Hasher>(&self, h: &mut H) {
		match self {
			Literal::Null => (),
			Literal::Boolean(b) => b.hash(h),
			Literal::Number(n) => n.hash(h),
			Literal::String(s) => s.hash(h)
		}
	}
}

impl<J: Json> Literal<J> {
	pub fn as_str(&self) -> Option<&str> {
		match self {
			Literal::String(s) => Some(s.as_str()),
			_ => None
		}
	}

	pub fn as_bool(&self) -> Option<bool> {
		match self {
			Literal::Boolean(b) => Some(*b),
			_ => None
		}
	}

	pub fn as_number(&self) -> Option<&J::Number> {
		match self {
			Literal::Number(n) => Some(n),
			_ => None
		}
	}
}

/// Value object.
///
/// Either a typed literal value, or an internationalized language string.
#[derive(PartialEq, Eq, Clone)]
pub enum Value<J: Json, T: Id = IriBuf> {
	/// A typed value.
	Literal(Literal<J>, Option<T>),

	/// A language tagged string.
	LangString(LangString),

	/// A JSON literal value.
	Json(J)
}

impl<J: Json, T: Id> Value<J, T> {
	pub fn as_str(&self) -> Option<&str> {
		match self {
			Value::Literal(lit, _) => lit.as_str(),
			Value::LangString(str) => Some(str.as_str()),
			Value::Json(_) => None
		}
	}

	pub fn as_bool(&self) -> Option<bool> {
		match self {
			Value::Literal(lit, _) => lit.as_bool(),
			_ => None
		}
	}

	pub fn as_number(&self) -> Option<&J::Number> {
		match self {
			Value::Literal(lit, _) => lit.as_number(),
			_ => None
		}
	}

	/// Return the type of the value if any.
	///
	/// This will return `Some(Type::Json)` for JSON literal values.
	pub fn typ(&self) -> Option<Type<&T>> {
		match self {
			Value::Literal(_, Some(ty)) => Some(Type::Ref(ty)),
			Value::Json(_) => Some(Type::Json),
			_ => None
		}
	}

	/// If the value is a language tagged string, return its associated language if any.
	///
	/// Returns `None` if the value is not a language tagged string.
	pub fn language(&self) -> Option<LanguageTag> {
		match self {
			Value::LangString(tag) => tag.language(),
			_ => None
		}
	}

	/// If the value is a language tagged string, return its associated direction if any.
	///
	/// Returns `None` if the value is not a language tagged string.
	pub fn direction(&self) -> Option<Direction> {
		match self {
			Value::LangString(str) => str.direction(),
			_ => None
		}
	}
}

impl<J: Json, T: Id> object::Any<J, T> for Value<J, T> {
	fn as_ref(&self) -> object::Ref<J, T> {
		object::Ref::Value(self)
	}
}

impl<J: Json, T: Id> Hash for Value<J, T> {
	fn hash<H: Hasher>(&self, h: &mut H) {
		match self {
			Value::Literal(lit, ty) => {
				lit.hash(h);
				ty.hash(h);
			},
			Value::LangString(str) => str.hash(h),
			Value::Json(json) => json.hash(h)
		}
	}
}

impl<J: Json, T: Id> AsJson<J> for Value<J, T> {
	fn as_json(&self) -> J {
		let mut obj = J::Object::default();

		match self {
			Value::Literal(lit, ty) => {
				match lit {
					Literal::Null => {
						obj.insert(Keyword::Value.into_str().into(), json::Value::Null.into());
					},
					Literal::Boolean(b) => {
						obj.insert(Keyword::Value.into_str().into(), b.as_json());
					},
					Literal::Number(n) => {
						obj.insert(Keyword::Value.into_str().into(), json::Value::Number(n.clone()).into());
					},
					Literal::String(s) => {
						obj.insert(Keyword::Value.into_str().into(), s.as_json());
					}
				}

				if let Some(ty) = ty {
					obj.insert(Keyword::Type.into_str().into(), ty.as_json());
				}
			},
			Value::LangString(str) => {
				obj.insert(Keyword::Value.into_str().into(), str.as_str().as_json());

				if let Some(language) = str.language() {
					obj.insert(Keyword::Language.into_str().into(), language.as_json());
				}

				if let Some(direction) = str.direction() {
					obj.insert(Keyword::Direction.into_str().into(), direction.as_json());
				}
			},
			Value::Json(json) => {
				obj.insert(Keyword::Value.into_str().into(), json.clone());
				obj.insert(Keyword::Type.into_str().into(), Keyword::Json.into_str().as_json());
			}
		}

		json::Value::Object(obj).into()
	}
}
