use futures::{Future, Stream};
use actix_web::{web, HttpResponse, HttpRequest};
use serde_json::{self, Value, json};
use utils::{self, app, ImageInput, ImageOutputItem, ImageError};

#[allow(dead_code)]
pub fn route(req: HttpRequest, payload: web::Payload) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    macro_rules! image_upload_all {
        ($items:expr) => (
            {
                boxed!(
                    futures::stream::iter_ok::<Vec<Result<ImageInput, ImageError>>, std::io::Error>($items)
                        .and_then(|item_result| {
                            (match item_result {
                                Ok(item) => app::image_upload(item),
                                Err(err) => {
                                    Box::new(futures::future::err::<ImageOutputItem, ImageError>(
                                        err
                                    ))
                                }
                            })
                            .then(|r| {
                                Ok::<Result<ImageOutputItem, ImageError>, std::io::Error>(r)
                            })
                        })
                        .fold(Vec::new(), |mut items, item| {
                            items.push(item);
                            Ok::<_, std::io::Error>(items)
                        })
                        .map(|items| json!({
                            "items": items
                        }))
                        .then(utils::to_response)
                )
            }
        );
    }

    let max_size = match utils::get_content_length(&req.headers(), app::get_json_max_size()) {
        Ok(size) => size,
        _ => {
            return future_wrap!(utils::response_payload_too_large());
        }
    } as usize;

    boxed!(
        payload
        .map_err(|_err| {
            //eprintln!("IO error: {:?}", err);
            actix_web::Error::from(utils::response_bad_request())
        })
        .fold(Vec::with_capacity(max_size as usize), move |mut body, chunk| {
            if (body.len() + chunk.len()) > max_size {
                Err(actix_web::Error::from(utils::response_payload_too_large()))
            } else {
                body.extend_from_slice(&chunk);
                Ok(body)
            }
        })
        .and_then(|body| {
            if let Ok(input) = serde_json::from_slice::<Value>(&body) {
                match input {
                    Value::String(v) => {
                        if let Ok(item) = utils::data_url_parse(v) {
                            return boxed!(app::image_upload(item).then(to_format).then(utils::to_response));
                        }
                    },
                    Value::Object(mut obj) => {
                        if let Ok(item) = utils::json_value_to_upload_image_item(&mut obj) {
                            return boxed!(app::image_upload(item).then(to_format).then(utils::to_response))
                        }
                    },
                    Value::Array(items) => {
                        return image_upload_all!(
                            items
                                .into_iter()
                                .map(|value| {
                                    match value {
                                        Value::String(v) => utils::data_url_parse(v),
                                        Value::Object(mut obj) => utils::json_value_to_upload_image_item(&mut obj),
                                        _ => Err(ImageError::Invalid)
                                    }
                                })
                                .collect()
                        );

                    },
                    _ => {}
                }
            }

            future_wrap!(utils::response_bad_request())
        })
    )
}

/// Конвертирует результат в условленный формат
#[inline]
pub fn to_format<A, E>(r: Result<A, E>) -> Result<Value, actix_web::Error>
where
    A: serde::Serialize,
    E: std::fmt::Debug
{
    Ok(match r {
        Ok(item) => json!({
            "items": [
                {
                    "Ok": item
                }
            ]
        }),
        Err(err) => {
            eprintln!("Error: {:?}", err);
            json!({
                "items": [
                    {
                        "Err": {
                            "type": "Invalid"
                        }
                    }
                 ]
            })
        }
    })
}
