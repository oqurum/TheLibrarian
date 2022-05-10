mod auth;

pub use auth::login::LoginPage;

pub mod home;
pub mod options;
pub mod media_view;
pub mod list_authors;
pub mod author_view;

pub use home::HomePage;
pub use options::OptionsPage;
pub use media_view::MediaView;
pub use list_authors::AuthorListPage;
pub use author_view::AuthorView;