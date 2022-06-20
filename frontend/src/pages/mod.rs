mod auth;

pub use auth::login::LoginPage;

pub mod home;
pub mod options;
pub mod book_view;
pub mod list_authors;
pub mod list_edits;
pub mod author_view;

pub use home::HomePage;
pub use options::OptionsPage;
pub use book_view::BookView;
pub use list_authors::AuthorListPage;
pub use list_edits::EditListPage;
pub use author_view::AuthorView;