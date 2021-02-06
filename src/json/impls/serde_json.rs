use crate::json::{
	Json,
	Value,
	ValueRef,
	ValueMut
};

impl Json for serde_json::Value {
	type MetaData = ();
	
	type Number = serde_json::Number;

	type Array = Vec<serde_json::Value>;

	type Key = String;

	type Object = serde_json::Map<String, serde_json::Value>;

	fn metadata(&self) -> &Self::MetaData {
		&()
	}
	
	fn as_ref(&self) -> ValueRef<Self> {
		panic!("TODO")
	}

	fn as_mut(&mut self) -> ValueMut<Self> {
		panic!("TODO")
	}
}