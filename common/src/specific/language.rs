use crate::util::string_to_upper_case;




pub static LANGUAGES: [&str; 1] = [
	"english",
];



pub fn get_language_id(value: &str) -> Option<usize> {
	let value = value.to_lowercase();
	LANGUAGES.iter().position(|v| *v == value)
}


pub fn get_language_name(value: u16) -> Option<String> {
	LANGUAGES.get(value as usize).map(|v| string_to_upper_case(v.to_string()))
}