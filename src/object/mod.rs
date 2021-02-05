//! Nodes, lists and values.

pub mod value;
pub mod node;

use std::collections::HashSet;
use std::hash::Hash;
use std::fmt;
use iref::{Iri, IriBuf};
use langtag::LanguageTag;
use crate::{
	json::{
		self,
		Json,
		AsJson
	},
	Id,
	Lenient,
	Reference,
	Indexed,
	syntax::Keyword
};

pub use value::{
	Literal,
	Value
};
pub use node::Node;

pub trait Any<J: Json, T: Id>: AsJson {
	fn as_ref(&self) -> Ref<T>;

	#[inline]
	fn id(&self) -> Option<&Lenient<Reference<T>>> {
		match self.as_ref() {
			Ref::Node(n) => n.id.as_ref(),
			_ => None
		}
	}

	#[inline]
	fn language<'a>(&'a self) -> Option<LanguageTag> where T: 'a {
		match self.as_ref() {
			Ref::Value(value) => value.language(),
			_ => None
		}
	}

	#[inline]
	fn is_value(&self) -> bool {
		match self.as_ref() {
			Ref::Value(_) => true,
			_ => false
		}
	}

	#[inline]
	fn is_node(&self) -> bool {
		match self.as_ref() {
			Ref::Node(_) => true,
			_ => false
		}
	}

	#[inline]
	fn is_graph(&self) -> bool {
		match self.as_ref() {
			Ref::Node(n) => n.is_graph(),
			_ => false
		}
	}

	#[inline]
	fn is_list(&self) -> bool {
		match self.as_ref() {
			Ref::List(_) => true,
			_ => false
		}
	}
}

/// Object reference.
pub enum Ref<'a, J: Json, T: Id> {
	/// Value object.
	Value(&'a Value<J, T>),

	/// Node object.
	Node(&'a Node<J, T>),

	/// List object.
	List(&'a [Indexed<Object<J, T>>])
}

/// Object.
///
/// JSON-LD connects together multiple kinds of data objects.
/// Objects may be nodes, values or lists of objects.
#[derive(PartialEq, Eq, Hash)]
pub enum Object<J: Json, T: Id = IriBuf> {
	/// Value object.
	Value(Value<T>),

	/// Node object.
	Node(Node<T>),

	/// List object.
	List(Vec<Indexed<Object<J, T>>>),
}

impl<J: Json, T: Id> Object<J, T> {
	/// Identifier of the object, if it is a node object.
	pub fn id(&self) -> Option<&Lenient<Reference<T>>> {
		match self {
			Object::Node(n) => n.id.as_ref(),
			_ => None
		}
	}

	/// Identifier of the object as an IRI.
	///
	/// If the object is a node identified by an IRI, returns this IRI.
	/// Returns `None` otherwise.
	pub fn as_iri(&self) -> Option<Iri> {
		match self {
			Object::Node(node) => node.as_iri(),
			_ => None
		}
	}

	/// Tests if the object is a value.
	pub fn is_value(&self) -> bool {
		match self {
			Object::Value(_) => true,
			_ => false
		}
	}

	/// Tests if the object is a node.
	pub fn is_node(&self) -> bool {
		match self {
			Object::Node(_) => true,
			_ => false
		}
	}

	/// Tests if the object is a graph object (a node with a `@graph` field).
	pub fn is_graph(&self) -> bool {
		match self {
			Object::Node(n) => n.is_graph(),
			_ => false
		}
	}

	/// Tests if the object is a list.
	pub fn is_list(&self) -> bool {
		match self {
			Object::List(_) => true,
			_ => false
		}
	}

	/// Get the object as a string.
	///
	/// If the object is a value that is a string, returns this string.
	/// If the object is a node that is identified, returns the identifier as a string.
	/// Returns `None` otherwise.
	pub fn as_str(&self) -> Option<&str> {
		match self {
			Object::Value(value) => value.as_str(),
			Object::Node(node) => node.as_str(),
			_ => None
		}
	}

	/// Get the value as a boolean, if it is.
	pub fn as_bool(&self) -> Option<bool> {
		match self {
			Object::Value(value) => value.as_bool(),
			_ => None
		}
	}

	/// Get the value as a number, if it is.
	pub fn as_number(&self) -> Option<J::Number> {
		match self {
			Object::Value(value) => value.as_number(),
			_ => None
		}
	}

	/// Try to convert this object into an unnamed graph.
	pub fn into_unnamed_graph(self: Indexed<Self>) -> Result<HashSet<Indexed<Object<T>>>, Indexed<Self>> {
		let (obj, index) = self.into_parts();
		match obj {
			Object::Node(n) => {
				match n.into_unnamed_graph() {
					Ok(g) => Ok(g),
					Err(n) => Err(Indexed::new(Object::Node(n), index))
				}
			},
			obj => Err(Indexed::new(obj, index))
		}
	}

	/// If the objat is a language-tagged value,
	/// Return its associated language.
	pub fn language(&self) -> Option<LanguageTag> {
		match self {
			Object::Value(value) => value.language(),
			_ => None
		}
	}
}

impl<J: Json, T: Id> Any<T> for Object<J, T> {
	fn as_ref(&self) -> Ref<J, T> {
		match self {
			Object::Value(value) => Ref::Value(value),
			Object::Node(node) => Ref::Node(node),
			Object::List(list) => Ref::List(list.as_ref())
		}
	}
}

impl<J: Json, T: Id> fmt::Debug for Object<J, T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.as_json().pretty(2))
	}
}

impl<J: Json, T: Id> From<Value<J, T>> for Object<J, T> {
	fn from(value: Value<T>) -> Object<T> {
		Object::Value(value)
	}
}

impl<J: Json, T: Id> From<Node<J, T>> for Object<J, T> {
	fn from(node: Node<T>) -> Object<T> {
		Object::Node(node)
	}
}

impl<J: Json, T: Id> AsJson<J> for Object<J, T> {
	fn as_json(&self) -> J {
		match self {
			Object::Value(v) => v.as_json(),
			Object::Node(n) => n.as_json(),
			Object::List(items) => {
				let mut obj = J::Object::new();
				obj.insert(Keyword::List.into(), items.as_json());
				json::Value::Object(obj)
			}
		}
	}
}
