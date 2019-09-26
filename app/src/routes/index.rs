#[allow(dead_code)]
#[inline]
pub fn route_index() -> impl actix_web::Responder {
    actix_web::HttpResponse::Ok()
        .content_type("text/plain; charset=utf-8")
        .body("Home")
}

#[allow(dead_code)]
#[inline]
pub fn provide() -> actix_web::Route {
    actix_web::web::get().to(route_index)
}
