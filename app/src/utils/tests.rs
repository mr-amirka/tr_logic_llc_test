#[cfg(test)]

extern crate test;

use super::*;
use futures_cpupool::{CpuPool};
use base64;

use crate::utils;

#[allow(dead_code)]
const TEST_TEMPORARY_DIR: &'static str = "./test_tmp/";

const TEST_BASE64_IMAGE: &'static str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABgAAAAYCAMAAADXqc3KAAAB+FBMVEUAAAA/mUPidDHiLi5Cn0XkNTPmeUrkdUg/m0Q0pEfcpSbwaVdKskg+lUP4zA/iLi3msSHkOjVAmETdJSjtYFE/lkPnRj3sWUs8kkLeqCVIq0fxvhXqUkbVmSjwa1n1yBLepyX1xxP0xRXqUkboST9KukpHpUbuvRrzrhF/ljbwaljuZFM4jELaoSdLtElJrUj1xxP6zwzfqSU4i0HYnydMtUlIqUfywxb60AxZqEXaoifgMCXptR9MtklHpEY2iUHWnSjvvRr70QujkC+pUC/90glMuEnlOjVMt0j70QriLS1LtEnnRj3qUUXfIidOjsxAhcZFo0bjNDH0xxNLr0dIrUdmntVTkMoyfL8jcLBRuErhJyrgKyb4zA/5zg3tYFBBmUTmQTnhMinruBzvvhnxwxZ/st+Ktt5zp9hqota2vtK6y9FemNBblc9HiMiTtMbFtsM6gcPV2r6dwroseLrMrbQrdLGdyKoobKbo3Zh+ynrgVllZulTsXE3rV0pIqUf42UVUo0JyjEHoS0HmsiHRGR/lmRz/1hjqnxjvpRWfwtOhusaz0LRGf7FEfbDVmqHXlJeW0pbXq5bec3fX0nTnzmuJuWvhoFFhm0FtrziBsjaAaDCYWC+uSi6jQS3FsSfLJiTirCOkuCG1KiG+wSC+GBvgyhTszQ64Z77KAAAARXRSTlMAIQRDLyUgCwsE6ebm5ubg2dLR0byXl4FDQzU1NDEuLSUgC+vr6urq6ubb29vb2tra2tG8vLu7u7uXl5eXgYGBgYGBLiUALabIAAABsElEQVQoz12S9VPjQBxHt8VaOA6HE+AOzv1wd7pJk5I2adpCC7RUcHd3d3fXf5PvLkxheD++z+yb7GSRlwD/+Hj/APQCZWxM5M+goF+RMbHK594v+tPoiN1uHxkt+xzt9+R9wnRTZZQpXQ0T5uP1IQxToyOAZiQu5HEpjeA4SWIoksRxNiGC1tRZJ4LNxgHgnU5nJZBDvuDdl8lzQRBsQ+s9PZt7s7Pz8wsL39/DkIfZ4xlB2Gqsq62ta9oxVlVrNZpihFRpGO9fzQw1ms0NDWZz07iGkJmIFH8xxkc3a/WWlubmFkv9AB2SEpDvKxbjidN2faseaNV3zoHXvv7wMODJdkOHAegweAfFPx4G67KluxzottCU9n8CUqXzcIQdXOytAHqXxomvykhEKN9EFutG22p//0rbNvHVxiJywa8yS2KDfV1dfbu31H8jF1RHiTKtWYeHxUvq3bn0pyjCRaiRU6aDO+gb3aEfEeVNsDgm8zzLy9egPa7Qt8TSJdwhjplk06HH43ZNJ3s91KKCHQ5x4sw1fRGYDZ0n1L4FKb9/BP5JLYxToheoFCVxz57PPS8UhhEpLBVeAAAAAElFTkSuQmCC";
lazy_static! {
    // parse input value
    static ref TEST_INPUT_IMAGE_ITEM: utils::ImageInput = utils::data_url_parse_inner(TEST_BASE64_IMAGE.into()).unwrap();
    static ref TEST_IMAGE_DATA: Bytes = Bytes::from(base64::decode(TEST_INPUT_IMAGE_ITEM.content.as_ref()).unwrap());
    static ref TEST_CPU_POOL: CpuPool = CpuPool::new(1);
}


#[test]
fn it_get_file_path_by_hex() {

    let hex_name = "6568c9e9c35a7fa06f236e89f704d8c9b47183a24f2c978dba6c92e2747e3a13";
    let extension = ".png";
    let (file_path, subdir_path) = utils::get_file_path_by_hex(&hex_name, extension);

    assert_eq!(file_path, std::path::PathBuf::from_str("Z/W/j/J6cNaf6BvI26J9wTYybRxg6JPLJeNumyS4nR-OhM.png").unwrap());
    assert_eq!(subdir_path, std::path::PathBuf::from_str("Z/W/j/").unwrap());
}


#[test]
fn it_image_data_url_parse_inner() {
    assert_eq!(TEST_INPUT_IMAGE_ITEM.name, None);
    assert_eq!(TEST_INPUT_IMAGE_ITEM.r#type, mime::IMAGE_PNG);
}

#[test]
fn it_image_mat_to_data() {
    let image_mat = image::data_to_mat(&TEST_IMAGE_DATA[..]).unwrap();

    assert_eq!(image_mat.size().unwrap(), opencv::core::Size {
        width: 24,
        height: 24,
    });
}

#[test]
fn it_image_data_to_mat() {
    let image_mat_src = image::data_to_mat(&TEST_IMAGE_DATA[..]).unwrap();
    let image_data_vec_of_uchar = image::mat_to_data(&image_mat_src, ".png").unwrap();
    let image_mat_dst = image::data_to_mat(image_data_vec_of_uchar.to_slice()).unwrap();
    let image_mat_dst_size = image_mat_dst.size().unwrap();

    assert_eq!(image_mat_dst_size, opencv::core::Size {
        width: 24,
        height: 24,
    });
    assert_eq!(
        image_mat_src.size().unwrap(),
        image_mat_dst_size
    );
}

#[test]
fn it_image_mat_resize() {
    let image_mat = image::data_to_mat(&TEST_IMAGE_DATA[..]).unwrap();
    let image_mat_target = image::mat_resize(&image_mat, 12, 12).unwrap();

    assert_eq!(image_mat_target.size().unwrap(), opencv::core::Size {
        width: 12,
        height: 12,
    });
}

#[test]
fn it_image_is_invalid_format() {
    assert_eq!(image::is_invalid_format(&TEST_IMAGE_DATA[..]), false);
    assert_eq!(image::is_invalid_format(b"Invalid value"), true);
}

#[test]
fn it_image_get_content_type_by_signature() {
    assert_eq!(image::get_content_type_by_signature(&TEST_IMAGE_DATA[..]), Some("image/png"));
    assert_eq!(image::get_content_type_by_signature(b"Invalid value"), None);
}

#[test]
fn it_hex_to_base64() {
    assert_eq!(utils::hex_to_base64("000001"), "AAAB");
    assert_eq!(utils::hex_to_base64("000002"), "AAAC");
    assert_eq!(utils::hex_to_base64("00000F"), "AAAP");
    assert_eq!(utils::hex_to_base64("000FFF"), "AA__");
}

#[test]
fn it_is_url_data() {
    assert_eq!(utils::is_url_data(TEST_BASE64_IMAGE), true);
    assert_eq!(utils::is_url_data("Invalid value"), false);
}

#[test]
fn it_data_url_parse() {
    assert_eq!(utils::data_url_parse(TEST_BASE64_IMAGE.into()), Ok::<_,utils::ImageError>(TEST_INPUT_IMAGE_ITEM.clone()));
    assert_eq!(utils::data_url_parse("Invalid value".into()), Err(utils::ImageError::Invalid));
}


#[test]
fn it_json_value_to_upload_image_item() {
    use serde_json::{Value, json};

    let content = "iVBORw0KGgoAAAANSUhEUgAAABgAAAAYCAMAAADXqc3KAAAB+FBMVEUAAAA/mUPidDHiLi5Cn0XkNTPmeUrkdUg/m0Q0pEfcpSbwaVdKskg+lUP4zA/iLi3msSHkOjVAmETdJSjtYFE/lkPnRj3sWUs8kkLeqCVIq0fxvhXqUkbVmSjwa1n1yBLepyX1xxP0xRXqUkboST9KukpHpUbuvRrzrhF/ljbwaljuZFM4jELaoSdLtElJrUj1xxP6zwzfqSU4i0HYnydMtUlIqUfywxb60AxZqEXaoifgMCXptR9MtklHpEY2iUHWnSjvvRr70QujkC+pUC/90glMuEnlOjVMt0j70QriLS1LtEnnRj3qUUXfIidOjsxAhcZFo0bjNDH0xxNLr0dIrUdmntVTkMoyfL8jcLBRuErhJyrgKyb4zA/5zg3tYFBBmUTmQTnhMinruBzvvhnxwxZ/st+Ktt5zp9hqota2vtK6y9FemNBblc9HiMiTtMbFtsM6gcPV2r6dwroseLrMrbQrdLGdyKoobKbo3Zh+ynrgVllZulTsXE3rV0pIqUf42UVUo0JyjEHoS0HmsiHRGR/lmRz/1hjqnxjvpRWfwtOhusaz0LRGf7FEfbDVmqHXlJeW0pbXq5bec3fX0nTnzmuJuWvhoFFhm0FtrziBsjaAaDCYWC+uSi6jQS3FsSfLJiTirCOkuCG1KiG+wSC+GBvgyhTszQ64Z77KAAAARXRSTlMAIQRDLyUgCwsE6ebm5ubg2dLR0byXl4FDQzU1NDEuLSUgC+vr6urq6ubb29vb2tra2tG8vLu7u7uXl5eXgYGBgYGBLiUALabIAAABsElEQVQoz12S9VPjQBxHt8VaOA6HE+AOzv1wd7pJk5I2adpCC7RUcHd3d3fXf5PvLkxheD++z+yb7GSRlwD/+Hj/APQCZWxM5M+goF+RMbHK594v+tPoiN1uHxkt+xzt9+R9wnRTZZQpXQ0T5uP1IQxToyOAZiQu5HEpjeA4SWIoksRxNiGC1tRZJ4LNxgHgnU5nJZBDvuDdl8lzQRBsQ+s9PZt7s7Pz8wsL39/DkIfZ4xlB2Gqsq62ta9oxVlVrNZpihFRpGO9fzQw1ms0NDWZz07iGkJmIFH8xxkc3a/WWlubmFkv9AB2SEpDvKxbjidN2faseaNV3zoHXvv7wMODJdkOHAegweAfFPx4G67KluxzottCU9n8CUqXzcIQdXOytAHqXxomvykhEKN9EFutG22p//0rbNvHVxiJywa8yS2KDfV1dfbu31H8jF1RHiTKtWYeHxUvq3bn0pyjCRaiRU6aDO+gb3aEfEeVNsDgm8zzLy9egPa7Qt8TSJdwhjplk06HH43ZNJ3s91KKCHQ5x4sw1fRGYDZ0n1L4FKb9/BP5JLYxToheoFCVxz57PPS8UhhEpLBVeAAAAAElFTkSuQmCC";

    let json_image_item = json!({
        "name": "any-name.png",
        "type": "image/png",
        "content": content
    });

    let mut object_value = if let Value::Object(object_value) = json_image_item {
        object_value
    } else {
        panic!();
    };

    assert_eq!(
        utils::json_value_to_upload_image_item(&mut object_value),
        Ok(ImageInput {
            name: Some("any-name.png".into()),
            r#type: mime::IMAGE_PNG,
            content: Bytes::from(content),
        })
    );
}

#[test]
fn it_json_value_to_upload_image_item_with_error() {
    use serde_json::{Value, json};

    let json_image_item = json!({
        "name": "any-name.png",
        "type": "image/png"
    });

    let mut object_value = if let Value::Object(object_value) = json_image_item {
        object_value
    } else {
        panic!();
    };

    assert_eq!(
        utils::json_value_to_upload_image_item(&mut object_value),
        Err(utils::ImageError::Invalid)
    );
}


#[test]
fn it_response_bad_request() {
    use serde_json::{json, Value};
    use actix_web::{http::{StatusCode, header}, dev};

    let resp = utils::response_bad_request();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        serde_json::from_slice::<Value>({

            let body = if let dev::ResponseBody::Body(body) = resp.body() {
                body
            } else {
                panic!();
            };

            let body_bytes = if let dev::Body::Bytes(body_bytes) = body {
                body_bytes
            } else {
                panic!();
            };

            &body_bytes[..]

        }).unwrap(),
        json!({
            "error": {
                "type": "Invalid"
            }
        })
    );
    assert_eq!(
        resp.headers().get(header::CONTENT_TYPE).unwrap(),
        header::HeaderValue::from_static("application/json")
    );

}


#[test]
fn it_response_payload_too_large() {
    use serde_json::{json, Value};
    use actix_web::{http::{StatusCode, header}, dev};

    let resp = utils::response_payload_too_large();

    assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(
        serde_json::from_slice::<Value>({

            let body = if let dev::ResponseBody::Body(body) = resp.body() {
                body
            } else {
                panic!();
            };

            let body_bytes = if let dev::Body::Bytes(body_bytes) = body {
                body_bytes
            } else {
                panic!();
            };

            &body_bytes[..]

        }).unwrap(),
        json!({
            "error": {
                "type": "Overflow"
            }
        })
    );
    assert_eq!(
        resp.headers().get(header::CONTENT_TYPE).unwrap(),
        header::HeaderValue::from_static("application/json")
    );

}

#[test]
fn it_response_unsupported_media_type() {
    use serde_json::{json, Value};
    use actix_web::{http::{StatusCode, header}, dev};

    let resp = utils::response_unsupported_media_type();

    assert_eq!(resp.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert_eq!(
        serde_json::from_slice::<Value>({

            let body = if let dev::ResponseBody::Body(body) = resp.body() {
                body
            } else {
                panic!();
            };

            let body_bytes = if let dev::Body::Bytes(body_bytes) = body {
                body_bytes
            } else {
                panic!();
            };

            &body_bytes[..]

        }).unwrap(),
        json!({
            "error": {
                "type": "Unsupported"
            }
        })
    );
    assert_eq!(
        resp.headers().get(header::CONTENT_TYPE).unwrap(),
        header::HeaderValue::from_static("application/json")
    );

}

#[test]
fn it_get_content_length() {
    use actix_web::http::header::{HeaderMap, HeaderName, HeaderValue};

    let max_size: u64 = 1024;
    let mut headers = HeaderMap::new();

    headers.insert(
        HeaderName::from_bytes(b"content-length").unwrap(),
        HeaderValue::from_bytes(b"100").unwrap()
    );

    assert_eq!(
        utils::get_content_length(&headers, max_size),
        Ok(100 as u64)
    );
}

#[test]
fn it_get_content_length_with_overflow() {
    use actix_web::http::header::{HeaderMap, HeaderName, HeaderValue};

    let max_size: u64 = 1024;
    let mut headers = HeaderMap::new();

    headers.insert(
        HeaderName::from_bytes(b"content-length").unwrap(),
        HeaderValue::from_bytes(b"2000").unwrap()
    );

    assert_eq!(
        utils::get_content_length(&headers, max_size),
        Err(())
    );
}

#[test]
fn it_get_content_length_with_none() {
    use actix_web::http::header::{HeaderMap};

    let max_size: u64 = 1024;
    let headers = HeaderMap::new();

    assert_eq!(
        utils::get_content_length(&headers, max_size),
        Ok(1024 as u64)
    );
}

#[test]
fn it_to_response() {
    use serde_json::{json, Value};
    use actix_web::{http::{StatusCode, header}, dev};


    let resp = utils::to_response(Ok::<_, utils::ImageError>(utils::ImageOutput {
        name: "image.png".into(),
        size: 1024,
    })).unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        serde_json::from_slice::<Value>({

            let body = if let dev::ResponseBody::Body(body) = resp.body() {
                body
            } else {
                panic!();
            };

            let body_bytes = if let dev::Body::Bytes(body_bytes) = body {
                body_bytes
            } else {
                panic!();
            };

            &body_bytes[..]

        }).unwrap(),
        json!({
            "name": "image.png",
            "size": 1024,
        })
    );
    assert_eq!(
        resp.headers().get(header::CONTENT_TYPE).unwrap(),
        header::HeaderValue::from_static("application/json")
    );

}

#[test]
fn it_to_response_with_error() {
    use serde_json::{json, Value};
    use actix_web::{http::{StatusCode, header}, dev};


    let resp = utils::to_response(Err::<utils::ImageOutput, _>(utils::ImageError::Invalid)).unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        serde_json::from_slice::<Value>({

            let body = if let dev::ResponseBody::Body(body) = resp.body() {
                body
            } else {
                panic!();
            };

            let body_bytes = if let dev::Body::Bytes(body_bytes) = body {
                body_bytes
            } else {
                panic!();
            };

            &body_bytes[..]

        }).unwrap(),
        json!({
            "error": {
                "type": "Invalid"
            }
        })
    );
    assert_eq!(
        resp.headers().get(header::CONTENT_TYPE).unwrap(),
        header::HeaderValue::from_static("application/json")
    );
}

#[test]
fn it_stream_to_sink() {
    use crypto;
    use crypto::digest::Digest;
    use futures::Future;

    std::fs::create_dir_all(TEST_TEMPORARY_DIR).unwrap();

    let stream = futures::stream::once::<_, std::io::Error>(
        Ok(TEST_IMAGE_DATA.clone())
    );

    let test_file_path: std::path::PathBuf = format!("{}it_stream_to_sink.png", TEST_TEMPORARY_DIR).into();

    let async_file_sink = async_fs::AsyncFileSink::from_std(
        &TEST_CPU_POOL,
        std::fs::File::create(&test_file_path).unwrap(),
    );
    let size_limit: u64 = 10240;

    let (_, _, mut encryptor, file_size) = utils::stream_to_sink(
        stream,
        async_file_sink,
        crypto::sha2::Sha256::new(),
        size_limit,
    ).wait().unwrap();

    std::fs::remove_file(test_file_path).unwrap();

    assert_eq!(file_size, 1086);
    assert_eq!(encryptor.result_str(), "6568c9e9c35a7fa06f236e89f704d8c9b47183a24f2c978dba6c92e2747e3a13");
}
