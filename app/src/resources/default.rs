use crate::routes;

pub fn provide() -> actix_web::Resource {
    routes::set_default_routes(actix_web::web::resource("/{path:.*}"))
}
