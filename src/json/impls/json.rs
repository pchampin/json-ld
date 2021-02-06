impl crate::json::Json for ::json::JsonValue {
	type MetaData = ();
	
	type Number = ::json::number::Number;

	type Array = Vec<::json::JsonValue>;

	type Key: // Key is hidden in the current version...

	type Object = ::json::object::Object;

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