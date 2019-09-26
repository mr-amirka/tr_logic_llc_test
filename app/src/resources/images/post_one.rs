use futures::{ Future, Stream };
use actix_web::{
    web,
    HttpResponse,
    HttpRequest,
    HttpMessage,
};
use serde_json::{json};
use async_fs;
use utils::{self, app};


#[allow(dead_code)]
pub fn route(req: HttpRequest, payload: web::Payload) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {

    let mime_type = match req.mime_type() {
        Ok(mime_type_option) => {
            match mime_type_option {
                Some(mime_type) => mime_type,
                None => {
                    return future_wrap!(utils::response_bad_request());
                }
            }
        },
        Err(_err) => {
            //eprintln!("Error: {:?}", err);
            return future_wrap!(utils::response_bad_request());
        }
    };

    let max_size = match utils::get_content_length(&req.headers(), app::get_file_max_size()) {
        Ok(size) => size,
        _ => {
            return future_wrap!(utils::response_payload_too_large());
        }
    };


    if let Some(extension) = utils::image::get_supported_extension_by_mime_name(mime_type.subtype()) {
        return boxed!(
            //app::upload_file(
            app::upload_image(
                &async_fs::DEFAULT_CPU_POOL,
                payload.map_err(|_err| {
                    //eprintln!("Error: {:?}", err);
                    std::io::Error::new(std::io::ErrorKind::Other, "Payload error")
                }),
                extension,
                max_size,
            )
            .then(|r| {
                Ok(match r {
                    Ok((name, size)) => HttpResponse::Ok().json(json!({
                        "items": [
                            {
                                "Ok": {
                                    "name": name,
                                    "size": size,
                                }
                            }
                        ]
                    })),
                    Err(err) => {
                        //eprintln!("Error: {:?}", err);
                        if err.kind() == std::io::ErrorKind::UnexpectedEof {
                            utils::response_payload_too_large()
                        } else {
                            utils::response_bad_request()
                        }
                    }
                })
            })
        );
    }

    eprintln!("This mime type is not supported: {:?}", mime_type);
    future_wrap!(utils::response_unsupported_media_type())
}
