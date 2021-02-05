use cc_traits::{
	Len,
	MapMut,
	Get,
	Iter,
	VecMut,
	WithCapacity,
};

mod build;
pub use build::AsJson;

pub trait Json: Clone + PartialEq + From<Value<Self>> + for<'a> From<ValueRef<'a, Self>> + Sync + Send {
	type MetaData: Clone;
	
	type Number: PartialEq;

	type Array: Clone + cc_traits::VecMut<Self> + WithCapacity + for<'a> Iter<'a, Item=&'a Self>;

	type Key: Ord + AsRef<str> + for<'a> From<&'a str>;
	
	type Object: Clone + Len + WithCapacity + MapMut<Self::Key, Self> + for<'a> Get<&'a str> + for<'a> Iter<'a, Item=(&'a Self::Key, &'a Self)>;

	fn metadata(&self) -> &Self::MetaData;
	
	fn as_ref(&self) -> ValueRef<Self>;

	fn is_null(&self) -> bool {
		self.as_ref().is_null()
	}

	fn is_string(&self) -> bool {
		self.as_ref().is_string()
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

// pub trait JsonMut: Json {
// 	type ArrayMut<'a>;
// 	type ObjectMut<'a>;

// 	fn as_mut(&mut self) -> Meta<ValueMut<Self>, M>;
// }

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
	Number(T::Number),
	String(&'a str),
	Array(&'a T::Array),
	Object(&'a T::Object)
}

impl<'a, T: 'a + Json> ValueRef<'a, T> {
	pub fn is_null(&self) -> bool {
		match self {
			ValueRef::Null => true,
			_ => false
		}
	}

	pub fn is_string(&self) -> bool {
		match self {
			ValueRef::String(_) => true,
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