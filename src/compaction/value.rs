use cc_traits::MapInsert;
use crate::{
	Id,
	ContextMut,
	Value,
	Error,
	context::{
		self,
		Loader,
		Local,
		Inversible
	},
	syntax::{
		Keyword,
		Container,
		ContainerType,
		Type
	},
	json::{
		self,
		Json,
		AsJson
	}
};
use super::{
	Options,
	compact_iri
};

/// Compact the given indexed value.
pub async fn compact_indexed_value_with<J: Json, T: Id, C: ContextMut<T>, L: Loader>(value: &Value<J, T>, index: Option<&str>, active_context: Inversible<T, &C>, active_property: Option<&str>, loader: &mut L, options: Options) -> Result<J, Error> where C::LocalContext: From<L::Output> {
	// If the term definition for active property in active context has a local context:
	let mut active_context = active_context.into_borrowed();
	if let Some(active_property) = active_property {
		if let Some(active_property_definition) = active_context.get(active_property) {
			if let Some(local_context) = &active_property_definition.context {
				active_context = Inversible::new(local_context.process_with(*active_context.as_ref(), loader, active_property_definition.base_url(), context::ProcessingOptions::from(options).with_override()).await?.into_inner()).into_owned()
			}
		}
	}

	// If element has an @value or @id entry and the result of using the Value Compaction algorithm,
	// passing active context, active property, and element as value is a scalar,
	// or the term definition for active property has a type mapping of @json,
	// return that result.

	// Here starts the Value Compaction Algorithm.

	// Initialize result to a copy of value.
	let mut result = J::Object::default();

	// If the active context has a null inverse context,
	// set inverse context in active context to the result of calling the
	// Inverse Context Creation algorithm using active context.
	// NOTE never null here (FIXME is that true?)

	// Initialize inverse context to the value of inverse context in active context.
	// DONE

	let active_property_definition = match active_property {
		Some(active_property) => active_context.get(active_property),
		None => None
	};

	// Initialize language to the language mapping for active property in active context,
	// if any, otherwise to the default language of active context.
	let language = match active_property_definition {
		Some(def) => match def.language.as_ref() {
			Some(lang) => lang.as_ref().map(|l| l.as_ref()).option(),
			None => active_context.default_language()
		},
		None => active_context.default_language()
	};

	// Initialize direction to the direction mapping for active property in active context,
	// if any, otherwise to the default base direction of active context.
	let direction = match active_property_definition {
		Some(def) => match def.direction {
			Some(dir) => dir.option(),
			None => active_context.default_base_direction()
		},
		None => active_context.default_base_direction()
	};

	// If value has an @id entry and has no other entries other than @index:
	// NOTE not possible here

	// Otherwise, if value has an @type entry whose value matches the type mapping of
	// active property, set result to the value associated with the @value entry of value.
	let type_mapping: Option<Type<&T>> = match active_property_definition {
		Some(def) => def.typ.as_ref().map(|t| t.into()),
		None => None
	};

	let container_mapping = match active_property_definition {
		Some(def) => def.container,
		None => Container::None
	};

	let remove_index = (index.is_some() && container_mapping.contains(ContainerType::Index)) || index.is_none();

	match value {
		Value::Literal(lit, ty) => {
			use crate::object::value::Literal;
			if ty.as_ref().map(|t| Type::Ref(t)) == type_mapping && remove_index {
				match lit {
					Literal::Null => return Ok(json::Value::Null.into()),
					Literal::Boolean(b) => return Ok(b.as_json()),
					Literal::Number(n) => return Ok(json::Value::Number(n.clone()).into()),
					Literal::String(s) => {
						if ty.is_some() || (language.is_none() && direction.is_none()) {
							return Ok(s.as_json())
						} else {
							let compact_key: J = compact_iri(active_context.as_ref(), Keyword::Value, true, false, options)?;
							result.insert(compact_key.as_str().unwrap().into(), s.as_json());
						}
					}
				}
			} else {
				let compact_key: J = compact_iri(active_context.as_ref(), Keyword::Value, true, false, options)?;
				match lit {
					Literal::Null => {
						result.insert(compact_key.as_str().unwrap().into(), json::Value::Null.into());
					},
					Literal::Boolean(b) => {
						result.insert(compact_key.as_str().unwrap().into(), b.as_json());
					},
					Literal::Number(n) => {
						result.insert(compact_key.as_str().unwrap().into(), json::Value::Number(n.clone()).into());
					},
					Literal::String(s) => {
						result.insert(compact_key.as_str().unwrap().into(), s.as_json());
					}
				}

				if let Some(ty) = ty {
					let compact_key: J = compact_iri(active_context.as_ref(), Keyword::Type, true, false, options)?;
					let compact_ty = compact_iri(active_context.as_ref(), ty, true, false, options)?;
					result.insert(compact_key.as_str().unwrap().into(), compact_ty);
				}
			}
		},
		Value::LangString(ls) => {
			let ls_language = ls.language();//.map(|l| Nullable::Some(l));
			let ls_direction = ls.direction();//.map(|d| Nullable::Some(d));

			if remove_index
			&& (ls_language.is_none() || language == ls_language) // || (ls.language().is_none() && language.is_none()))
			&& (ls_direction.is_none() || direction == ls_direction) { // || (ls.direction().is_none() && direction.is_none())) {
				return Ok(ls.as_str().as_json())
			} else {
				let compact_key: J = compact_iri(active_context.as_ref(), Keyword::Value, true, false, options)?;
				result.insert(compact_key.as_str().unwrap().into(), ls.as_str().as_json());

				if let Some(language) = ls.language() {
					let compact_key: J = compact_iri(active_context.as_ref(), Keyword::Language, true, false, options)?;
					result.insert(compact_key.as_str().unwrap().into(), language.as_json());
				}

				if let Some(direction) = ls.direction() {
					let compact_key: J = compact_iri(active_context.as_ref(), Keyword::Direction, true, false, options)?;
					result.insert(compact_key.as_str().unwrap().into(), direction.as_json());
				}
			}
		},
		Value::Json(value) => {
			if type_mapping == Some(Type::Json) && remove_index {
				return Ok(value.clone())
			} else {
				let compact_key: J = compact_iri(active_context.as_ref(), Keyword::Value, true, false, options)?;
				result.insert(compact_key.as_str().unwrap().into(), value.clone());

				let compact_key: J = compact_iri(active_context.as_ref(), Keyword::Type, true, false, options)?;
				let compact_ty = compact_iri(active_context.as_ref(), Keyword::Json, true, false, options)?;
				result.insert(compact_key.as_str().unwrap().into(), compact_ty);
			}
		}
	}

	if !remove_index {
		if let Some(index) = index {
			let compact_key: J = compact_iri(active_context.as_ref(), Keyword::Index, true, false, options)?;
			result.insert(compact_key.as_str().unwrap().into(), index.as_json());
		}
	}

	Ok(json::Value::Object(result).into())
}