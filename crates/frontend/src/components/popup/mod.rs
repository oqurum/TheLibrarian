pub mod edit_metadata;
pub mod search;
pub mod search_person;


pub use search::PopupSearch;
pub use search_person::PopupSearchPerson;


#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SearchBy {
    External,
    Local,
}