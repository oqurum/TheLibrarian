use actix_web::{web, Scope, dev::{ServiceFactory, ServiceRequest, ServiceResponse}, HttpResponse};

pub mod book;
pub mod edit;
pub mod external;
pub mod member;
pub mod person;
pub mod publisher;
pub mod poster;
pub mod tag;
pub mod settings;

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
        // Book
        .service(book::add_new_book)
        .service(book::load_book_list)
        .service(book::get_book_info)
        .service(book::update_book_id)
        .service(book::load_book_thumbnail)

        // Tags
        .service(tag::get_tags)
        .service(tag::create_new_tag)
        .service(tag::get_tag_by_id)

        // Book Tags
        .service(tag::get_book_tag)
        .service(tag::add_book_tag)
        .service(tag::delete_book_tag)
        .service(tag::get_tags_for_book_id)

        // Member
        .service(member::load_member_self)

        // Person
        .service(person::load_author_list)
        .service(person::load_person)
        .service(person::load_person_thumbnail)

        // Poster
        .service(poster::get_local_image)
        .service(poster::get_poster_list)
        .service(poster::post_change_poster)
        .service(poster::post_upload_poster)

        // Edit
        .service(edit::load_edit_list)
        .service(edit::load_edit)
        .service(edit::update_edit)

        // External
        .service(external::get_external_search)
        .service(external::get_external_item)

        // Settings
        .service(settings::get_settings)
        .service(settings::update_settings)

        .default_service(web::route().to(default_handler))
}

async fn default_handler() -> HttpResponse {
    HttpResponse::NotFound().finish()
}