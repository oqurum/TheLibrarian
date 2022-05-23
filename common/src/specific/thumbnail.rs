use serde::{Deserialize, Deserializer, Serialize, Serializer};


pub static MISSING_THUMB_PATH: &str = "/images/missingthumbnail.jpg";


#[derive(Debug, Clone, PartialEq)]
pub enum ThumbnailStore {
	Path(String),
	None
}

impl ThumbnailStore {
	pub fn is_none(&self) -> bool {
		matches!(self, Self::None)
	}

	pub fn is_some(&self) -> bool {
		!self.is_none()
	}

	pub fn as_url(&self) -> String {
		match self {
			Self::None => String::from(MISSING_THUMB_PATH),
			_ => format!("/api/v1/image/{}", self.as_value()),
		}
	}


	pub fn as_value(&self) -> &str {
		match self {
			Self::Path(v) => v.as_str(),
			_ => unreachable!("Self::as_value()"),
		}
	}

	pub fn into_value(self) -> String {
		match self {
			Self::Path(v) => v,
			_ => unreachable!("Self::into_value()"),
		}
	}

	pub fn to_optional_string(&self) -> Option<String> {
		if self.is_some() {
			Some(self.to_string())
		} else {
			None
		}
	}
}


impl ToString for ThumbnailStore {
	fn to_string(&self) -> String {
		self.as_value().to_string()
	}
}

impl From<&str> for ThumbnailStore {
	fn from(value: &str) -> Self {
		Self::Path(value.to_string())
	}
}

impl From<String> for ThumbnailStore {
	fn from(value: String) -> Self {
		Self::Path(value)
	}
}


impl From<Option<String>> for ThumbnailStore {
	fn from(value: Option<String>) -> Self {
		value.map(|v| v.into()).unwrap_or(Self::None)
	}
}

impl From<Option<&str>> for ThumbnailStore {
	fn from(value: Option<&str>) -> Self {
		value.map(|v| v.into()).unwrap_or(Self::None)
	}
}


impl<'de> Deserialize<'de> for ThumbnailStore {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de> {
		Ok(Option::<String>::deserialize(deserializer)?.into())
	}
}

impl Serialize for ThumbnailStore {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer {
		if self.is_some() {
			serializer.serialize_str(&self.to_string())
		} else {
			serializer.serialize_none()
		}
	}
}