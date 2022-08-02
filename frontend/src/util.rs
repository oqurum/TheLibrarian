use web_sys::Element;


/// Truncate string based off of char indices instead of bytes.
pub fn truncate_on_indices(s: &mut String, max_chars: usize) {
    if let Some((new_len, _)) = s.char_indices().nth(max_chars) {
        s.truncate(new_len);
    }
}




pub enum LoadingItem<V> {
    Loading,
    Loaded(V),
}


pub fn does_parent_contain_class(element: &Element, value: &str) -> bool {
    if element.class_list().contains(value) {
        true
    } else if let Some(element) = element.parent_element() {
        does_parent_contain_class(&element, value)
    } else {
        false
    }
}