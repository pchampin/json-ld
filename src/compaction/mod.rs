use std::collections::HashSet;
use futures::future::{LocalBoxFuture, FutureExt};
use cc_traits::{
	Len,
	Get,
	GetMut,
	PushBack,
	MapInsert
};
use crate::{
	Id,
	ContextMut,
	Indexed,
	object,
	Object,
	Value,
	Error,
	ProcessingMode,
	context::{
		self,
		Loader,
		Local,
		inverse::{
			Inversible,
			TypeSelection,
			LangSelection
		}
	},
	syntax::{
		Keyword,
		ContainerType,
		Term
	},
	json::{
		self,
		Json,
		AsJson
	}
};

mod iri;
mod node;
mod value;
mod property;

pub(crate) use iri::*;
use node::*;
use value::*;
use property::*;

#[derive(Clone, Copy)]
pub struct Options {
	pub processing_mode: ProcessingMode,
	pub compact_to_relative: bool,
	pub compact_arrays: bool,
	pub ordered: bool
}

impl From<Options> for context::ProcessingOptions {
	fn from(options: Options) -> context::ProcessingOptions {
		let mut opt = context::ProcessingOptions::default();
		opt.processing_mode = options.processing_mode;
		opt
	}
}

impl From<crate::expansion::Options> for Options {
	fn from(options: crate::expansion::Options) -> Options {
		Options {
			processing_mode: options.processing_mode,
			ordered: options.ordered,
			..Options::default()
		}
	}
}

impl Default for Options {
	fn default() -> Options {
		Options {
			processing_mode: ProcessingMode::default(),
			compact_to_relative: true,
			compact_arrays: true,
			ordered: false
		}
	}
}

pub trait Compact<J: Json, T: Id> {
	fn compact_with<'a, C: ContextMut<T>, L: Loader>(&'a self, active_context: Inversible<T, &'a C>, type_scoped_context: Inversible<T, &'a C>, active_property: Option<&'a str>, loader: &'a mut L, options: Options) -> LocalBoxFuture<'a, Result<J, Error>> where J: 'a, T:'a, C::LocalContext: From<L::Output>;

	fn compact<'a, C: ContextMut<T>, L: Loader>(&'a self, active_context: Inversible<T, &'a C>, loader: &'a mut L) -> LocalBoxFuture<'a, Result<J, Error>> where J: 'a, T: 'a, C::LocalContext: From<L::Output> {
		async move {
			self.compact_with(active_context.clone(), active_context, None, loader, Options::default()).await
		}.boxed_local()
	}
}

enum TypeLangValue<'a, T: Id> {
	Type(TypeSelection<T>),
	Lang(LangSelection<'a>)
}

pub trait CompactIndexed<J: Json, T: Id> {
	fn compact_indexed_with<'a, C: ContextMut<T>, L: Loader>(&'a self, index: Option<&'a str>, active_context: Inversible<T, &'a C>, type_scoped_context: Inversible<T, &'a C>, active_property: Option<&'a str>, loader: &'a mut L, options: Options) -> LocalBoxFuture<'a, Result<J, Error>> where J: 'a, T: 'a, C::LocalContext: From<L::Output>;
}

impl<T: Id, J: Json, V: CompactIndexed<J, T>> Compact<J, T> for Indexed<V> {
	fn compact_with<'a, C: ContextMut<T>, L: Loader>(&'a self, active_context: Inversible<T, &'a C>, type_scoped_context: Inversible<T, &'a C>, active_property: Option<&'a str>, loader: &'a mut L, options: Options) -> LocalBoxFuture<'a, Result<J, Error>> where J: 'a, T: 'a, C::LocalContext: From<L::Output> {
		self.inner().compact_indexed_with(self.index(), active_context, type_scoped_context, active_property, loader, options)
	}
}

impl<T: Id, J: Json, N: object::Any<J, T>> CompactIndexed<J, T> for N {
	fn compact_indexed_with<'a, C: ContextMut<T>, L: Loader>(&'a self, index: Option<&'a str>, active_context: Inversible<T, &'a C>, type_scoped_context: Inversible<T, &'a C>, active_property: Option<&'a str>, loader: &'a mut L, options: Options) -> LocalBoxFuture<'a, Result<J, Error>> where J: 'a, T: 'a, C::LocalContext: From<L::Output> {
		match self.as_ref() {
			object::Ref::Value(value) => async move {
				compact_indexed_value_with(value, index, active_context, active_property, loader, options).await
			}.boxed_local(),
			object::Ref::Node(node) => async move {
				compact_indexed_node_with(node, index, active_context, type_scoped_context, active_property, loader, options).await
			}.boxed_local(),
			object::Ref::List(list) => async move {
				let mut active_context = active_context;
				// If active context has a previous context, the active context is not propagated.
				// If element does not contain an @value entry, and element does not consist of
				// a single @id entry, set active context to previous context from active context,
				// as the scope of a term-scoped context does not apply when processing new node objects.
				if let Some(previous_context) = active_context.previous_context() {
					active_context = Inversible::new(previous_context)
				}

				// If the term definition for active property in active context has a local context:
				// FIXME https://github.com/w3c/json-ld-api/issues/502
				//       Seems that the term definition should be looked up in `type_scoped_context`.
				let mut active_context = active_context.into_borrowed();
				let mut list_container = false;
				if let Some(active_property) = active_property {
					if let Some(active_property_definition) = type_scoped_context.get(active_property) {
						if let Some(local_context) = &active_property_definition.context {
							active_context = Inversible::new(local_context.process_with(*active_context.as_ref(), loader, active_property_definition.base_url(), context::ProcessingOptions::from(options).with_override()).await?.into_inner()).into_owned()
						}

						list_container = active_property_definition.container.contains(ContainerType::List);
					}
				}

				if list_container {
					compact_collection_with(list.iter(), active_context.as_ref(), active_context.as_ref(), active_property, loader, options).await
				} else {
					let mut result = J::Object::default();
					compact_property(&mut result, Term::Keyword(Keyword::List), list, active_context.as_ref(), loader, false, options).await?;

					// If expanded property is @index and active property has a container mapping in
					// active context that includes @index,
					if let Some(index) = index {
						let mut index_container = false;
						if let Some(active_property) = active_property {
							if let Some(active_property_definition) = active_context.get(active_property) {
								if active_property_definition.container.contains(ContainerType::Index) {
									// then the compacted result will be inside of an @index container,
									// drop the @index entry by continuing to the next expanded property.
									index_container = true;
								}
							}
						}

						if !index_container {
							// Initialize alias by IRI compacting expanded property.
							let alias: J = compact_iri(active_context.as_ref(), Keyword::Index, true, false, options)?;

							// Add an entry alias to result whose value is set to expanded value and continue with the next expanded property.
							result.insert(alias.as_str().unwrap().into(), index.as_json());
						}
					}

					Ok(json::Value::Object(result).into())
				}
			}.boxed_local()
		}
	}
}


/// Default value of `as_array` is false.
fn add_value<J: Json>(map: &mut J::Object, key: &str, value: J, as_array: bool) {
	match map.get(key).map(J::as_ref) {
		Some(json::ValueRef::Array(_)) => (),
		Some(original_value) => {
			let value: J = original_value.into();
			let mut array = J::Array::default();
			array.push_back(value);
			map.insert(key.into(), json::Value::Array(array).into());
		},
		None if as_array => {
			map.insert(key.into(), json::Value::Array(J::Array::default()).into());
		},
		None => ()
	}

	match value.into() {
		json::Value::Array(values) => {
			for value in values {
				add_value(map, key, value, false)
			}
		},
		value => {
			match map.get_mut(key).map(J::as_mut) {
				Some(json::ValueMut::Array(values)) => {
					values.push_back(value.into());
				},
				Some(_) => unreachable!(),
				None => {
					map.insert(key.into(), value.into());
				}
			}
		}
	}
}

/// Get the `@value` field of a value object.
fn value_value<J: Json, T: Id>(value: &Value<J, T>) -> J {
	use crate::object::value::Literal;
	match value {
		Value::Literal(lit, _ty) => {
			match lit {
				Literal::Null => json::Value::Null.into(),
				Literal::Boolean(b) => b.as_json(),
				Literal::Number(n) => json::Value::Number(n.clone()).into(),
				Literal::String(s) => s.as_json()
			}
		},
		Value::LangString(str) => json::ValueRef::String(str.as_str()).into(),
		Value::Json(json) => json.clone()
	}
}

fn compact_collection_with<'a, J: 'a + Json, T: 'a + Id, O: 'a + Iterator<Item=&'a Indexed<Object<J, T>>>, C: ContextMut<T>, L: Loader>(items: O, active_context: Inversible<T, &'a C>, type_scoped_context: Inversible<T, &'a C>, active_property: Option<&'a str>, loader: &'a mut L, options: Options) -> LocalBoxFuture<'a, Result<J, Error>> where C::LocalContext: From<L::Output> {
	async move {
		let mut result = J::Array::default();

		for item in items {
			let compacted_item = item.compact_with(active_context.clone(), type_scoped_context.clone(), active_property, loader, options).await?;
			if !compacted_item.is_null() {
				result.push_back(compacted_item);
			}
		}

		let mut list_or_set = false;
		if let Some(active_property) = active_property {
			if let Some(active_property_definition) = active_context.get(active_property) {
				list_or_set = active_property_definition.container.contains(ContainerType::List) || active_property_definition.container.contains(ContainerType::Set);
			}
		}

		if result.is_empty()
		|| result.len() > 1
		|| !options.compact_arrays
		|| active_property == Some("@graph") || active_property == Some("@set")
		|| list_or_set {
			return Ok(json::Value::Array(result).into())
		}

		return Ok(result.into_iter().next().unwrap())
	}.boxed_local()
}

impl<J: Json, T: Id> Compact<J, T> for HashSet<Indexed<Object<J, T>>> {
	fn compact_with<'a, C: ContextMut<T>, L: Loader>(&'a self, active_context: Inversible<T, &'a C>, type_scoped_context: Inversible<T, &'a C>, active_property: Option<&'a str>, loader: &'a mut L, options: Options) -> LocalBoxFuture<'a, Result<J, Error>> where T: 'a, C::LocalContext: From<L::Output> {
		compact_collection_with(self.iter(), active_context, type_scoped_context, active_property, loader, options)
	}
}
