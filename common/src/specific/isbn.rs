

// Used to help handle ids a little better "amazon:{id}", "amazon_uk:{id}", "goodreads:{id}", "isbn:{id}", "google:{id}", "uuid:{id}", "urn:uuid:{id}", "urn:isbn:{id}"
pub fn parse_book_id(value: &str) -> IdType {
	if let Some((prefix, suffix)) = value.rsplit_once(':') {
		let prefix = prefix.to_lowercase().replace(' ', "");
		let suffix = suffix.trim().to_owned();

		match prefix.as_str() {
			"urn:isbn" |
			"isbn" => IdType::Isbn(suffix),

			"urn:uuid" |
			"uuid" => IdType::Uuid(suffix),

			_ => IdType::UnknownKeyValue(prefix, suffix),
		}
	} else {
		IdType::UnknownValue(value.trim().to_string())
	}
}

pub enum IdType {
	Isbn(String),
	Uuid(String),

	UnknownKeyValue(String, String),
	UnknownValue(String)
}

impl IdType {
	pub fn get_possible_isbn_value(&self) -> Option<&str> {
		match self {
			Self::UnknownValue(v) if v.chars().all(|v| ('0'..='9').contains(&v)) => Some(v.as_str()),
			Self::Isbn(v) => Some(v.as_str()),

			_ => None,
		}
	}

	pub fn into_possible_isbn_value(self) -> Option<String> {
		match self {
			Self::UnknownValue(v) if v.chars().all(|v| ('0'..='9').contains(&v)) => Some(v),
			Self::Isbn(v) => parse_isbn_13(&v).or_else(|| parse_isbn_10(&v)).or(Some(v)),

			_ => None,
		}
	}

	pub fn into_possible_single_value(self) -> Option<String> {
		match self {
			Self::Isbn(v) => Some(v),
			Self::Uuid(v) => Some(v),
			Self::UnknownKeyValue(_, v) => Some(v),
			Self::UnknownValue(v) => Some(v),
		}
	}


	/// Attempts to return an ISBN type of 13 or 12 in that order.
	pub fn as_isbn_13_or_10(&self) -> Option<String> {
		self.as_isbn_13().or_else(|| self.as_isbn_10())
	}

	pub fn as_isbn_13(&self) -> Option<String> {
		match self {
			Self::UnknownValue(v) => parse_isbn_13(v.as_str()),
			Self::Isbn(v) => parse_isbn_13(v.as_str()),

			_ => None,
		}
	}

	pub fn as_isbn_10(&self) -> Option<String> {
		match self {
			Self::UnknownValue(v) => parse_isbn_10(v.as_str()),
			Self::Isbn(v) => parse_isbn_10(v.as_str()),

			_ => None,
		}
	}
}

// TODO: Convert all ISBN-10's to ISBN-13

// TODO: Tests
pub fn parse_isbn_10(value: &str) -> Option<String> {
	let mut s = 0;
	let mut t = 0;

	let mut parse = value.split("").filter(|v| *v != "-" && !v.is_empty());

	let mut compiled = String::new();

	// Ensure first is a number.
	if let Some(v) = parse.next().filter(|v| v.parse::<usize>().is_ok()) {
		compiled.push_str(v);
	}

	for dig_str in parse.take(9) {
		let dig = match dig_str.parse::<usize>() {
			Ok(v) => v,
			Err(_) => return None
		};

		compiled.push_str(dig_str);

		t += dig;
		s += t;
	}

	Some(compiled).filter(|v| s != 0 && v.len() == 10 && (s % 11) == 0)
}

// TODO: Tests
pub fn parse_isbn_13(value: &str) -> Option<String> {
	let mut s = 0;

	let mut compiled = String::new();

	for (i, dig_str) in value.split("").filter(|v| *v != "-" && !v.is_empty()).take(13).enumerate() {
		let dig = match dig_str.parse::<usize>() {
			Ok(v) => v,
			Err(_) => return None
		};

		compiled.push_str(dig_str);

		let weight = if i % 2 == 0 { 1 } else { 3 };
		s += dig * weight;
	}

	Some(compiled).filter(|v| s != 0 && v.len() == 13 && (s % 10) == 0)
}