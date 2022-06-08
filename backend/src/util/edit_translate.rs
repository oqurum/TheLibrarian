




#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Output<V> {
	pub new_value: Option<V>,
	pub old_value: Option<V>,
}



pub fn cmp_opt_string(value_old: Option<String>, value_new: Option<String>) -> Output<String> {
	let old_exists = value_old.is_some();
	let both_equal = value_new == value_old;

	let mut old_out = None;
	let mut new_out = None;

	// If we have an old value AND (the new value doesn't exist OR the new value DOES NOT EQUAL the old value)
	// - Some(old) = OLD & NEW & NEQ

	if old_exists && value_new.is_some() && !both_equal {
		old_out = value_old;
	}

	// If we have an new value AND new val is not empty AND (the old value IS NULL OR new DOES NOT EQUAL old)
	// - Some(new) = NEW & !empty & !OLD
	// - Some(new) = NEW & !empty & NEQ

	// TODO: use new.is_some_with
	if value_new.is_some() && value_new.as_deref().map(|v| !v.is_empty()).unwrap_or_default() && (!old_exists || !both_equal) {
		new_out = value_new;
	}

	Output { new_value: new_out, old_value: old_out }
}

pub fn cmp_opt_number<V: std::cmp::PartialEq>(value_old: Option<V>, value_new: Option<V>) -> Output<V> {
	let old_exists = value_old.is_some();
	let both_equal = value_new == value_old;

	let mut old_out = None;
	let mut new_out = None;

	// If we have an old value AND (the new value doesn't exist OR the new value DOES NOT EQUAL the old value)
	// - Some(old) = OLD & NEW & NEQ

	if old_exists && value_new.is_some() && !both_equal {
		old_out = value_old;
	}

	// If we have an new value AND new val is not empty AND (the old value IS NULL OR new DOES NOT EQUAL old)
	// - Some(new) = NEW & !OLD
	// - Some(new) = NEW & NEQ

	if value_new.is_some() && (!old_exists || !both_equal) {
		new_out = value_new;
	}

	Output { new_value: new_out, old_value: old_out }
}

pub fn cmp_opt_bool(value_old: Option<bool>, value_new: Option<bool>) -> Output<bool> {
	let old_exists = value_old.is_some();
	let both_equal = value_new == value_old;

	let mut old_out = None;
	let mut new_out = None;

	// If we have an old value AND (the new value doesn't exist OR the new value DOES NOT EQUAL the old value)
	// - Some(old) = OLD & NEW & NEQ

	if old_exists && value_new.is_some() && !both_equal {
		old_out = value_old;
	}

	// If we have an new value AND new val is not empty AND (the old value IS NULL OR new DOES NOT EQUAL old)
	// - Some(new) = NEW & !OLD
	// - Some(new) = NEW & NEQ

	if value_new.is_some() && (!old_exists || !both_equal) {
		new_out = value_new;
	}

	Output { new_value: new_out, old_value: old_out }
}



#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_compare_opt_string() {
		// We have both a new and old value. We'll just output it.
		assert_eq!(
			cmp_opt_string(Some(String::from("Old")), Some(String::from("New"))),
			Output { new_value: Some(String::from("New")), old_value: Some(String::from("Old")) }
		);

		assert_eq!(
			cmp_opt_string(Some(String::from("Old")), Some(String::from("Old"))),
			Output { new_value: None, old_value: None }
		);

		assert_eq!(
			cmp_opt_string(Some(String::from("Old")), Some(String::from(""))),
			Output { new_value: None, old_value: Some(String::from("Old")) }
		);

		assert_eq!(
			cmp_opt_string(Some(String::from("Old")), None),
			Output { new_value: None, old_value: Some(String::from("Old")) }
		);

		assert_eq!(
			cmp_opt_string(None, Some(String::from("New"))),
			Output { new_value: Some(String::from("New")), old_value: None }
		);
	}
}