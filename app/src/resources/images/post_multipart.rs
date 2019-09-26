use futures::{ Future, Stream };
use actix_web::{HttpResponse};
use actix_multipart;
use async_fs;
use serde_json::{json};
use utils::{self, app, ImageOutputItem, ImageError};


#[allow(dead_code)]
pub fn route(payload: actix_multipart::Multipart) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    payload
        .map_err(|_err| {
            //eprintln!("Error: {:?}", err);
            std::io::Error::new(std::io::ErrorKind::Other, "Payload error")
        })
        .and_then(move |field| {

            if let Some(extension) = utils::image::get_supported_extension_by_mime_name(field.content_type().subtype()) {
               if let Some(content_disposition) = field.content_disposition() {
                    if let Some(name) = content_disposition.get_name() {
                        if name == "image" {
                            let original: Option<std::path::PathBuf> = match content_disposition.get_filename() {
                                Some(value) => Some(value.into()),
                                _ => None
                            };

                            return Box::new(
                                //app::upload_file(
                                app::upload_image(
                                    &async_fs::DEFAULT_CPU_POOL,
                                    field.map_err(|_err| {
                                        //eprintln!("Error: {:?}", err);
                                        std::io::Error::new(std::io::ErrorKind::Other, "Payload error")
                                    }),
                                    extension,
                                    app::get_file_max_size(),
                                )
                                .then(|r| {
                                    Ok(match r {
                                        Ok((name, size)) => Ok(ImageOutputItem {
                                            original,
                                            name,
                                            size,
                                        }),
                                        Err(err) => {
                                            if err.kind() == std::io::ErrorKind::UnexpectedEof {
                                                Err(ImageError::Overflow)
                                            } else {
                                                Err(ImageError::WriteError)
                                            }
                                        }
                                    })
                                })
                            ) as Box<dyn Future<Item = Result<ImageOutputItem, ImageError>, Error = std::io::Error>>
                        }
                    }
                }
           }

           Box::new(futures::future::ok(
              Err(ImageError::Unsupported)
           )) as Box<dyn Future<Item = Result<ImageOutputItem, ImageError>, Error = std::io::Error>>
        })
        .fold(Vec::new(), |mut items, item| {
            items.push(item);
            Ok::<_, std::io::Error>(items)
        })
        .map(|items| json!({
            "items": items
        }))
        .then(utils::to_response)
}
