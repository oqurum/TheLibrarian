use actix_web::{web, Scope, dev::{ServiceFactory, ServiceRequest, ServiceResponse}};

use super::LoginRequired;

pub mod book;
pub mod external;
pub mod member;
pub mod person;
pub mod publisher;

pub fn api_route() -> Scope<
	impl ServiceFactory<
		ServiceRequest,
		Config = (),
		Response = ServiceResponse<actix_web::body::BoxBody>,
		Error = actix_web::Error,
		InitError = (),
	>
> {
	web::scope("/api/v1")
		.wrap(LoginRequired)

		// Book
		.service(book::add_new_book)
		.service(book::load_book_list)
		.service(book::get_book_info)
		.service(book::load_book_thumbnail)

		// Member
		.service(member::load_member_self)

		// Person
		.service(person::load_author_list)
		.service(person::load_person_thumbnail)

		// External
		.service(external::get_external_search)
}