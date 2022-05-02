use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::Error;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Source {
	pub agent: String,
	pub value: String,
}

impl fmt::Display for Source {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}:{}", self.agent, self.value)
	}
}

impl TryFrom<&str> for Source {
	type Error = Error;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		let (source, value) = value.split_once(':')
			.ok_or(Error::SourceSplit)?;

		Ok(Self {
			agent: source.to_owned(),
			value: value.to_owned(),
		})
	}
}

impl TryFrom<String> for Source {
	type Error = Error;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		let (source, value) = value.split_once(':')
			.ok_or(Error::SourceSplit)?;

		Ok(Self {
			agent: source.to_owned(),
			value: value.to_owned(),
		})
	}
}

impl<'de> Deserialize<'de> for Source {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
		let resp = String::deserialize(deserializer)?;
		Ok(Self::try_from(resp).unwrap())
	}
}

impl Serialize for Source {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
		serializer.serialize_str(&self.to_string())
	}
}