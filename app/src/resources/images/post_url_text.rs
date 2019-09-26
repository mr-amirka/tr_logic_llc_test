use futures::{Future, Stream};
use actix_web::{
    web,
    HttpResponse,
    HttpRequest,
};
use utils::{self as utils, app};
use serde_json::{json};

#[allow(dead_code)]
pub fn route(req: HttpRequest, payload: web::Payload) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {

    match utils::get_content_length(&req.headers(), app::get_text_max_size()) {
        Ok(max_size) => boxed!(
            app::upload_url_data_image_by_lines_stream(
                payload.map_err(|_err| {
                    // eprintln!("Error: {:?}", err);
                    std::io::Error::new(std::io::ErrorKind::Other, "Payload error")
                }),
                max_size as usize
            )
            .map(|items| json!({
                "items": items
            }))
            .then(utils::to_response)
        ),
        _ => future_wrap!(utils::response_payload_too_large())
    }
}
