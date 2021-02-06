use iref::{Iri, IriBuf};
use langtag::LanguageTagBuf;
use crate::{
	Nullable,
	Id,
	Direction,
	syntax::{
		Term,
		Type,
		Container
	}
};

// A term definition.
#[derive(Clone)]
pub struct TermDefinition<J, T: Id> {
	// IRI mapping.
	pub value: Option<Term<T>>,

	// Prefix flag.
	pub prefix: bool,

	// Protected flag.
	pub protected: bool,

	// Reverse property flag.
	pub reverse_property: bool,

	// Optional base URL.
	pub base_url: Option<IriBuf>,

	// Optional context.
	pub context: Option<J>,

	// Container mapping.
	pub container: Container,

	// Optional direction mapping.
	pub direction: Option<Nullable<Direction>>,

	// Optional index mapping.
	pub index: Option<String>,

	// Optional language mapping.
	pub language: Option<Nullable<LanguageTagBuf>>,

	// Optional nest value.
	pub nest: Option<String>,

	// Optional type mapping.
	pub typ: Option<Type<T>>
}

impl<J, T: Id> TermDefinition<J, T> {
	pub fn base_url(&self) -> Option<Iri> {
		self.base_url.as_ref().map(|iri| iri.as_iri())
	}
}

impl<J, T: Id> Default for TermDefinition<J, T> {
	fn default() -> TermDefinition<J, T> {
		TermDefinition {
			value: None,
			prefix: false,
			protected: false,
			reverse_property: false,
			base_url: None,
			typ: None,
			language: None,
			direction: None,
			context: None,
			nest: None,
			index: None,
			container: Container::new()
		}
	}
}

impl<J: PartialEq, T: Id> PartialEq for TermDefinition<J, T> {
	fn eq(&self, other: &TermDefinition<J, T>) -> bool {
		// NOTE we ignore the `protected` flag.
		self.prefix == other.prefix &&
		self.reverse_property == other.reverse_property &&
		self.language == other.language &&
		self.direction == other.direction &&
		self.nest == other.nest &&
		self.index == other.index &&
		self.container == other.container &&
		self.base_url == other.base_url &&
		self.value == other.value &&
		self.typ == other.typ &&
		self.context == other.context
	}
}

impl<J: Eq, T: Id> Eq for TermDefinition<J, T> {}
