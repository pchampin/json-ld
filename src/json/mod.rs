
use std::hash::{
	Hash
};
use cc_traits::{
	Len,
	MapMut,
	Get,
	GetMut,
	Iter,
	WithCapacity,
	Remove,
	PopFront
};

mod build;
pub use build::AsJson;

mod impls;

pub trait Json: Clone + PartialEq + Eq + Hash + From<Value<Self>> + for<'a> From<ValueRef<'a, Self>> + Into<Value<Self>> + Send {
	type MetaData: Clone;
	
	type Number: Clone + PartialEq + PartialEq<i32> + Eq + Hash;

	type Array: Clone + Default + AsRef<[Self]> + cc_traits::VecMut<Self> + PopFront + WithCapacity + IntoIterator<Item=Self> + for<'a> Iter<'a, Item=&'a Self>;

	type Key: Clone + Ord + Hash + AsRef<str> + for<'a> From<&'a str>;

	type Object: Clone + Len + Default + WithCapacity + MapMut<Self::Key, Self> + for<'a> Get<&'a str> + for<'a> GetMut<&'a str> + for<'a> Remove<&'a str> + IntoIterator<Item=(Self::Key, Self)> + for<'a> Iter<'a, Item=(&'a Self::Key, &'a Self)>;

	fn metadata(&self) -> &Self::MetaData;
	
	fn as_ref(&self) -> ValueRef<Self>;

	fn as_mut(&mut self) -> ValueMut<Self>;

	fn is_null(&self) -> bool {
		self.as_ref().is_null()
	}

	fn is_empty(&self) -> bool {
		self.as_ref().is_empty()
	}

	fn is_string(&self) -> bool {
		self.as_ref().is_string()
	}

	fn is_array(&self) -> bool {
		self.as_ref().is_array()
	}

	fn is_object(&self) -> bool {
		self.as_ref().is_object()
	}

	fn as_bool(&self) -> Option<bool> {
		self.as_ref().as_bool()
	}
	
	fn as_str(&self) -> Option<&str> {
		self.as_ref().as_str()
	}

	fn as_number(&self) -> Option<&Self::Number> {
		self.as_ref().as_number()
	}

	fn as_object(&self) -> Option<&Self::Object> {
		self.as_ref().as_object()
	}

	fn get(&self, key: &str) -> Option<&Self> {
		match self.as_ref() {
			ValueRef::Object(obj) => obj.get(key),
			_ => None
		}
	}
}

pub enum Value<T: Json> {
	Null,
	Boolean(bool),
	Number(T::Number),
	String(String),
	Array(T::Array),
	Object(T::Object)
}

pub enum ValueRef<'a, T: Json> {
	Null,
	Boolean(bool),
	Number(&'a T::Number),
	String(&'a str),
	Array(&'a T::Array),
	Object(&'a T::Object)
}

pub enum ValueMut<'a, T: Json> {
	Null,
	Boolean(bool),
	Number(&'a mut T::Number),
	String(&'a mut str),
	Array(&'a mut T::Array),
	Object(&'a mut T::Object)
}

impl<'a, T: 'a + Json> ValueRef<'a, T> {
	pub fn is_null(&self) -> bool {
		match self {
			ValueRef::Null => true,
			_ => false
		}
	}

	pub fn is_empty(&self) -> bool {
		match *self {
			ValueRef::Null => true,
			ValueRef::Boolean(b) => !b,
			ValueRef::Number(n) => *n == 0i32,
			ValueRef::String(s) => s.is_empty(),
			ValueRef::Array(a) => a.is_empty(),
			ValueRef::Object(o) => o.is_empty()
		}
	}

	pub fn is_number(&self) -> bool {
		match self {
			ValueRef::Number(_) => true,
			_ => false
		}
	}

	pub fn is_string(&self) -> bool {
		match self {
			ValueRef::String(_) => true,
			_ => false
		}
	}

	pub fn is_array(&self) -> bool {
		match self {
			ValueRef::Array(_) => true,
			_ => false
		}
	}

	pub fn is_object(&self) -> bool {
		match self {
			ValueRef::Object(_) => true,
			_ => false
		}
	}

	pub fn as_bool(&self) -> Option<bool> {
		match self {
			ValueRef::Boolean(b) => Some(*b),
			_ => None
		}
	}

	pub fn as_number(&self) -> Option<&'a T::Number> {
		match self {
			ValueRef::Number(n) => Some(n),
			_ => None
		}
	}

	pub fn as_str(&self) -> Option<&'a str> {
		match self {
			ValueRef::String(s) => Some(s),
			_ => None
		}
	}

	pub fn as_object(&self) -> Option<&'a T::Object> {
		match self {
			ValueRef::Object(o) => Some(o),
			_ => None
		}
	}
}

impl<'s, 'a, T: 'a + Json> PartialEq<&'s str> for ValueRef<'a, T> {
	fn eq(&self, str: &&'s str) -> bool {
		match self {
			ValueRef::String(this) => this == str,
			_ => false
		}
	}
}

// pub fn hash_json<J: Json, H: Hasher>(value: &J, hasher: &mut H) {
// 	match value.as_ref() {
// 		ValueRef::Null => (),
// 		ValueRef::Boolean(b) => b.hash(hasher),
// 		ValueRef::Number(n) => n.hash(hasher),
// 		ValueRef::String(str) => str.hash(hasher),
// 		ValueRef::Array(ary) => {
// 			for item in ary.iter() {
// 				hash_json(item, hasher)
// 			}
// 		},
// 		ValueRef::Object(obj) => {
// 			// in JSON, the order of elements matters, so we don't need to be vigilant here.
// 			for (key, value) in obj.iter() {
// 				key.hash(hasher);
// 				hash_json(value, hasher);
// 			}
// 		}
// 	}
// }