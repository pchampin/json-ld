//! Expansion algorithm and types.

mod expanded;
mod iri;
mod literal;
mod value;
mod node;
mod array;
mod element;

use std::cmp::{Ord, Ordering};
use std::collections::HashSet;
use futures::Future;
use iref::{Iri, IriBuf};
use crate::{
	json::Json,
	ProcessingMode,
	Error,
	Id,
	Indexed,
	Object,
	ContextMut,
	context::{
		ProcessingOptions,
		Loader
	}
};

pub use expanded::*;
pub use iri::*;
pub use literal::*;
pub use value::*;
pub use node::*;
pub use array::*;
pub use element::*;

#[derive(Clone, Copy, Default)]
pub struct Options {
	/// Sets the processing mode.
	pub processing_mode: ProcessingMode,

	/// If true, an error is returned if a value fails to expand. If false, the value is dropped.
	pub strict: bool,

	/// If set to true, input document entries are processed lexicographically.
	/// If false, order is not considered in processing.
	pub ordered: bool
}

impl From<Options> for ProcessingOptions {
	fn from(options: Options) -> ProcessingOptions {
		let mut copt = ProcessingOptions::default();
		copt.processing_mode = options.processing_mode;
		copt
	}
}

impl From<crate::compaction::Options> for Options {
	fn from(options: crate::compaction::Options) -> Options {
		Options {
			processing_mode: options.processing_mode,
			ordered: options.ordered,
			..Options::default()
		}
	}
}

pub struct Entry<'a, T, J>(T, &'a J);

impl<'a, T: PartialEq, J> PartialEq for Entry<'a, T, J> {
	fn eq(&self, other: &Entry<'a, T, J>) -> bool {
		self.0.eq(&other.0)
	}
}

impl<'a, T: PartialOrd, J> PartialOrd for Entry<'a, T, J> {
	fn partial_cmp(&self, other: &Entry<'a, T, J>) -> Option<Ordering> {
		self.0.partial_cmp(&other.0)
	}
}

impl<'a, T: Eq, J> Eq for Entry<'a, T, J> {}

impl<'a, T: Ord, J> Ord for Entry<'a, T, J> {
	fn cmp(&self, other: &Entry<'a, T, J>) -> Ordering {
		self.0.cmp(&other.0)
	}
}

fn filter_top_level_item<J: Json, T: Id>(item: &Indexed<Object<J, T>>) -> bool {
	// Remove dangling values.
	match item.inner() {
		Object::Value(_) => false,
		_ => true
	}
}

pub fn expand<'a, J: Json, T: Send + Sync + Id, C: Send + Sync + ContextMut<T>, L: Send + Sync + Loader>(active_context: &'a C, element: &'a J, base_url: Option<Iri>, loader: &'a mut L, options: Options) -> impl 'a + Future<Output=Result<HashSet<Indexed<Object<J, T>>>, Error>> where C::LocalContext: Send + Sync + From<L::Output> + From<J>, L::Output: Into<J> {
	let base_url = base_url.map(|url| IriBuf::from(url));

	async move {
		let base_url = base_url.as_ref().map(|url| url.as_iri());
		let expanded = expand_element(active_context, None, element, base_url, loader, options, false).await?;
		if expanded.len() == 1 {
			match expanded.into_iter().next().unwrap().into_unnamed_graph() {
				Ok(graph) => Ok(graph),
				Err(obj) => {
					let mut set = HashSet::new();
					if filter_top_level_item(&obj) {
						set.insert(obj);
					}
					Ok(set)
				}
			}
		} else {
			Ok(expanded.into_iter().filter(filter_top_level_item).collect())
		}
	}
}
