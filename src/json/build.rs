use std::collections::HashSet;
use langtag::{
	LanguageTag,
	LanguageTagBuf
};
use cc_traits::{
	Len,
	Get,
	Iter,
	WithCapacity,
	PushBack
};
use super::Json;

pub trait AsJson<J: Json> {
	fn as_json(&self) -> J;
}

impl<J: Json> AsJson<J> for bool {
	fn as_json(&self) -> J {
		super::Value::Boolean(*self).into()
	}
}

impl<J: Json> AsJson<J> for str {
	fn as_json(&self) -> J {
		super::ValueRef::String(self).into()
	}
}

impl<J: Json> AsJson<J> for String {
	fn as_json(&self) -> J {
		super::ValueRef::String(self).into()
	}
}

impl<'a, J: Json, T: AsRef<[u8]> + ?Sized> AsJson<J> for LanguageTag<'a, T> {
	fn as_json(&self) -> J {
		self.as_str().as_json()
	}
}

impl<J: Json, T: AsRef<[u8]>> AsJson<J> for LanguageTagBuf<T> {
	fn as_json(&self) -> J {
		self.as_str().as_json()
	}
}

impl<J: Json, T: AsJson<J>> AsJson<J> for [T] {
	fn as_json(&self) -> J {
		let ary = J::Array::with_capacity(self.len());
		for item in self {
			ary.push_back(item.as_json());
		}

		super::Value::Array(ary).into()
	}
}

impl<J: Json, T: AsJson<J>> AsJson<J> for Vec<T> {
	fn as_json(&self) -> J {
		self.as_slice().as_json()
	}
}

impl<J: Json, T: AsJson> AsJson<J> for HashSet<T> {
	fn as_json(&self) -> J {
		let ary = J::Array::with_capacity(self.len());
		for item in self {
			ary.push_back(item.as_json());
		}

		super::Value::Array(ary).into()
	}
}

pub fn json_ld_eq<I: Json, J: Json>(a: &I, b: &J) -> bool where I::Number: PartialEq<J::Number>, J::Number: PartialEq<I::Number> {
	match (a.as_ref(), b.as_ref()) {
		(super::ValueRef::Array(a), super::ValueRef::Array(b)) if a.len() == b.len() => {
			let mut selected = Vec::with_capacity(a.len());
			selected.resize(a.len(), false);

			'a_items: for item in a.iter() {
				for i in 0..b.len() {
					if !selected[i] && json_ld_eq(&b[i], item) {
						selected[i] = true;
						continue 'a_items
					}
				}

				return false
			}

			true
		},
		(super::ValueRef::Object(a), super::ValueRef::Object(b)) if a.len() == b.len() => {
			for (key, value_a) in a.iter() {
				let key = key.as_ref();
				if let Some(value_b) = b.get(key) {
					if key == "@list" {
						match (value_a.as_ref(), value_b.as_ref()) {
							(super::ValueRef::Array(item_a), super::ValueRef::Array(item_b)) if item_a.len() == item_b.len() => {
								for i in 0..item_a.len() {
									if !json_ld_eq(&item_a[i], &item_b[i]) {
										return false
									}
								}
							},
							_ => {
								if !json_ld_eq(value_a, value_b) {
									return false
								}
							}
						}
					} else {
						if !json_ld_eq(value_a, value_b) {
							return false
						}
					}
				} else {
					return false
				}
			}

			true
		},
		(super::ValueRef::Null, super::ValueRef::Null) => true,
		(super::ValueRef::Number(a), super::ValueRef::Number(b)) => a == b,
		(super::ValueRef::String(a), super::ValueRef::String(b)) => a == b,
		_ => false
	}
}
