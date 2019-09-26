use actix_web::{web, HttpResponse, Responder, dev, guard, http, Result};
use actix_web::middleware::errhandlers::{ ErrorHandlerResponse };

pub mod index;

mod tests;


#[allow(dead_code)]
#[inline]
pub fn handle_500<B>(mut res: dev::ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    res.response_mut().headers_mut().insert(
        http::header::CONTENT_TYPE,
        http::HeaderValue::from_static("Error"),
    );
    Ok(ErrorHandlerResponse::Response(res))
}

#[allow(dead_code)]
#[inline]
pub fn route_404() -> impl Responder {
    HttpResponse::NotFound()
        .content_type("text/plain; charset=utf-8")
        .body("Not found")
}

#[allow(dead_code)]
#[inline]
pub fn route_method_not_allowed() -> impl Responder {
    HttpResponse::MethodNotAllowed()
        .content_type("text/plain; charset=utf-8")
        .body("Method Not Allowed")
}

#[allow(dead_code)]
#[inline]
pub fn route_options() -> impl Responder {
    HttpResponse::Ok()
}

#[allow(dead_code)]
#[inline]
pub fn set_default_routes(resource: actix_web::Resource) -> actix_web::Resource {
    resource
        .route(web::get().to(route_404))
        .route(
            web::route()
               .guard(guard::Options())
               .to(route_options)
        )
        .route(web::route().to(route_method_not_allowed))
}
