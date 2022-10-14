mod auth;

pub use auth::{authorize::AuthorizePage, login::LoginPage, logout::LogoutPage};

pub mod admin;
pub mod author_view;
pub mod book_view;
pub mod collection;
pub mod home;
pub mod list_authors;
pub mod list_collections;
pub mod list_edits;
pub mod options;

pub use author_view::AuthorView;
pub use book_view::BookView;
pub use collection::CollectionView;
pub use home::HomePage;
pub use list_authors::AuthorListPage;
pub use list_collections::ListCollectionsPage;
pub use list_edits::EditListPage;
pub use options::OptionsPage;
