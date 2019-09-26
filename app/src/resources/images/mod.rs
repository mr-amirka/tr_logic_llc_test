use actix_web::{
    web,
    guard,
    http,
};

use crate::routes;

pub mod post_one;
pub mod post_multipart;
pub mod post_json;
pub mod post_url_text;
pub mod get_one;

mod tests;

#[allow(dead_code)]
pub fn provide() -> actix_web::Resource {
    routes::set_default_routes(
        web::resource("/images")
            .route(
                web::route()
                    .guard(guard::fn_guard(|req| {
                        req.method == http::Method::POST && {
                            /*
                                Недоработка в Actix-Web в том, что для проверки соответствия запроса маршруту
                                приходится так каждый раз извлекать из мапы значение заголовка,
                                проседая тем самым в производительности.
                                Можно былоб часто используемые в логике данные (в частности значения некоторых заголовков)
                                передавать через поля в струкутре запроса.
                                # Examples:
                                ```ignore
                                if let Some(content_type) = req.content_type {
                                    ...
                                } else {
                                    false
                                }
                                ```
                            */
                            if let Some(content_type) = req.headers.get("content-type") {
                                content_type.as_bytes().starts_with(b"image/")
                            } else {
                                false
                            }
                        }
                    }))
                    .to_async(post_one::route)
            )
            .route(
                web::route()
                   .guard(guard::fn_guard(|req| {
                       req.method == http::Method::POST && {
                           if let Some(content_type) = req.headers.get("content-type") {
                               content_type.as_bytes().starts_with(b"multipart/form-data")
                           } else {
                               false
                           }
                       }
                   }))
                   .to_async(post_multipart::route)
            )
            .route(
                web::route()
                   .guard(guard::fn_guard(|req| {
                       req.method == http::Method::POST && {
                           if let Some(content_type) = req.headers.get("content-type") {
                               content_type.as_bytes().starts_with(b"application/json")
                           } else {
                               false
                           }
                       }
                   }))
                   .to_async(post_json::route)
            )
            .route(
                web::route()
                   .guard(guard::fn_guard(|req| {
                       req.method == http::Method::POST && {
                           if let Some(content_type) = req.headers.get("content-type") {
                               content_type.as_bytes().starts_with(b"text/plain")
                           } else {
                               false
                           }
                       }
                   }))
                   .to_async(post_url_text::route)
            )
    )
}
