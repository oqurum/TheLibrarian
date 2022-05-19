


pub static LANGUAGES: [&str; 1] = [
	"english",
];



pub fn get_language_id(value: &str) -> Option<usize> {
	let value = value.to_lowercase();
	LANGUAGES.iter().position(|v| *v == value)
}