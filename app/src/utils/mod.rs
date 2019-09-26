use futures::{self, Async};
use serde_derive::{Deserialize, Serialize};
use base64;
use actix_web::{
    HttpResponse,
    http::header::HeaderMap,
};
use serde_json::{Value, map::Map};
use bytes::Bytes;

use std::str::FromStr;

#[macro_use] pub mod app;
pub mod image;

mod app_tests;
mod tests;


/// С целью сокращение длины имени файла, его имя из HEX формата конвертируется в url-safe base64.
/// С целью облегчения чтения списка файлов каталога, когда их может быть очень много,
/// функция автоматически генерирует имена поддиректорий на основе трех первых символов имени файла.
/// #Example:
/// i2D7SQIQYCYlr9t4JMXgFw.png -> i/2/D/7SQIQYCYlr9t4JMXgFw.png
/// Пример использования:
/// let (
///     src_path, // полный путь к файлу
///     src_subdir // путь к директории файла
/// ) = utils::get_file_path_by_hex("6568c9e9c35a7fa06f236e89f704d8c9b47183a24f2c978dba6c92e2747e3a13", ".png");
pub fn get_file_path_by_hex(hex_name: &str, extension: &str)
    -> (std::path::PathBuf, std::path::PathBuf)
{
    let src_name = format!("{}{}", hex_to_base64(hex_name).trim_end_matches('='), extension);
    let target_subdir = std::path::PathBuf::from_str(&src_name[..1])
        .unwrap()
        .join(&src_name[1..2])
        .join(&src_name[2..3]);
    (target_subdir.join(&src_name[3..]), target_subdir)
}

/// Конвертирует хэш-сумму из heх представления в base64
#[inline]
pub fn hex_to_base64(hex: &str) -> String {
    let mut bytes = Vec::new();
    let len = hex.len();
    let mut i = 0;
    let mut next = 2;
    while i < len {
        match u8::from_str_radix(&hex[i..next], 16) {
            Ok(v) => bytes.push(v),
            Err(e) => eprintln!("Problem with hex: {}", e),
        }
        i = next;
        next += 2;
    }
    base64::encode_config(&bytes, base64::URL_SAFE)
}

pub const URL_DATA_PREFIX: &'static str = "data:";

/// Проверяет URL на наличие встроенного содержимого.
#[inline]
pub fn is_url_data(input: &str) -> bool {
    input.starts_with(URL_DATA_PREFIX)
}

/// Конвертирует строку URL со встроенным содержимым в ImageInput.
pub fn data_url_parse_inner(input: String) -> Result<ImageInput, ImageError> {
    let start_ptr = input.as_ptr() as usize;
    let end_ptr = start_ptr + input.len();

    let type_start = URL_DATA_PREFIX.as_bytes().len();
    let type_start_ptr = start_ptr + type_start;
    let mut offset_ptr = type_start_ptr;

    let mut type_length: usize = 0;
    while offset_ptr < end_ptr {
        if b';' == unsafe { *(offset_ptr as *const u8) } {
            type_length = offset_ptr - type_start_ptr;
            offset_ptr += 1;
            break;
        }
        offset_ptr += 1;
    }
    if type_length < 1 {
        return Err(ImageError::Invalid);
    }

    let mut content_start = 0;
    while offset_ptr < end_ptr {
        if b',' == unsafe { *(offset_ptr as *const u8) } {
            offset_ptr += 1;
            content_start = offset_ptr - start_ptr;
            break;
        }
        offset_ptr += 1;
    }

    if content_start < 1 {
        return Err(ImageError::Invalid);
    }

    if let Ok(mime_type_source) = std::str::from_utf8(unsafe {
        std::slice::from_raw_parts(type_start_ptr as *const u8, type_length)
    }) {
        if let Ok(mime_type) = mime_type_source.parse::<mime::Mime>() {
            return Ok(ImageInput {
                name: None,
                r#type: mime_type,
                content: Bytes::from(input).slice_from(content_start)
            });
        }
    }

    Err(ImageError::Invalid)
}

/// Конвертирует строку URL со встроенным содержимым в ImageInput.
#[inline]
pub fn data_url_parse(input: String) -> Result<ImageInput, ImageError> {
    if is_url_data(&input) {
        data_url_parse_inner(input)
    } else {
        Err(ImageError::Invalid)
    }
}

/// Конвертирует JSON объект в ImageInput.
#[inline]
pub fn json_value_to_upload_image_item(obj: &mut Map<String, Value>) -> Result<ImageInput, ImageError> {
    let name = match obj.remove("name") {
        Some(Value::String(name)) => Some(name),
        _ => None
    };
    if let Some(Value::String(mime_type)) = obj.remove("type") {
        if let Ok(mime_type) = mime_type.parse::<mime::Mime>() {
            if let Some(Value::String(content)) = obj.remove("content") {
                return Ok(ImageInput {
                    name,
                    r#type: mime_type,
                    content: Bytes::from(content),
                });
            }
        }
    }

    Err(ImageError::Invalid)
}


#[inline]
pub fn response_bad_request() -> HttpResponse {
    HttpResponse::BadRequest().json(ImageError::Invalid.get_serializable())
}

#[inline]
pub fn response_payload_too_large() -> HttpResponse {
    HttpResponse::PayloadTooLarge().json(ImageError::Overflow.get_serializable())
}

#[inline]
pub fn response_unsupported_media_type() -> HttpResponse {
    HttpResponse::UnsupportedMediaType().json(ImageError::Unsupported.get_serializable())
}

/// Извлекает числовое значение заголовка content-length, при условии, что оно не превышает max_size, иначе ошибка.
#[inline]
pub fn get_content_length(headers: &HeaderMap, mut max_size: u64) -> Result<u64, ()> {
    if let Some(v) = headers.get("content-length") {
        if let Ok(v) = v.to_str() {
            if let Ok(size) = u64::from_str_radix(v, 10) {
                if size > max_size {
                    return Err(());
                }
                max_size = size;
            }
        }
    }
    Ok(max_size)
}

/// Конвертирует результат в HttpResponse
#[inline]
pub fn to_response<A, E>(r: Result<A, E>) -> Result<HttpResponse, actix_web::Error>
where
    A: serde::Serialize,
    E: std::fmt::Debug
{
    Ok(match r {
        Ok(value) => HttpResponse::Ok().json(value),
        Err(err) => {
            eprintln!("Error: {:?}", err);
            response_bad_request()
        }
    })
}

/// Структура входных данных объекта изображения
#[derive(PartialEq, Debug, Clone)]
pub struct ImageInput {
    pub name: Option<String>,
    pub r#type: mime::Mime,
    pub content: Bytes, //base64 content
}

/// Возможные ошибки обработки изображения
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(tag = "type")]
pub enum ImageError {
    Invalid,
    Overflow,
    Unsupported,
    NetError,
    WriteError,
}


impl ImageError {
    pub fn get_serializable(self) -> ImageErrorWrapper {
        ImageErrorWrapper {
            error: self
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ImageErrorWrapper {
    pub error: ImageError,
}

/// Структура выходных данных объекта изображения
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ImageOutput {
    pub name: std::path::PathBuf,
    pub size: u64,
}

/// Структура выходных данных объекта изображения
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct ImageOutputItem {
    pub original: Option<std::path::PathBuf>,
    pub name: std::path::PathBuf,
    pub size: u64,
}

// StreamToSink

#[derive(Debug)]
pub struct StreamToSink<Stream, Sink, Encryptor> {
    state: Option<(Stream, Sink, Encryptor)>,
    bytes: Option<bytes::Bytes>,
    amt: u64,
    read_done: bool,
    size_limit: u64
}


/// Стримит данные c хэшированием и ограничением длины входных данных
pub fn stream_to_sink<Stream, Sink, Encryptor>(stream: Stream, sink: Sink, encryptor: Encryptor, size_limit: u64)
    -> StreamToSink<Stream, Sink, Encryptor>
where
    Stream: futures::stream::Stream<Item = bytes::Bytes, Error = std::io::Error>,
    Sink: futures::sink::Sink<SinkItem = bytes::Bytes, SinkError = std::io::Error>,
    Encryptor: crypto::digest::Digest,
{
    StreamToSink {
        state: Some((stream, sink, encryptor)),
        bytes: None,
        amt: 0,
        read_done: false,
        size_limit
    }
}

impl<Stream, Sink, Encryptor> futures::Future for StreamToSink<Stream, Sink, Encryptor>
where
    Stream: futures::stream::Stream<Item = bytes::Bytes, Error = std::io::Error>,
    Sink: futures::sink::Sink<SinkItem = bytes::Bytes, SinkError = std::io::Error>,
    Encryptor: crypto::digest::Digest,
{
    type Item = (Stream, Sink, Encryptor, u64);
    type Error = std::io::Error;

    fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
        let (stream, sink, encryptor) = self.state.as_mut().unwrap();

        loop {
            try_ready!(sink.poll_complete());

            if self.read_done {
                if self.bytes == None {
                    let (stream, sink, encryptor) = self.state.take().unwrap();
                    return Ok(Async::Ready((
                        stream,
                        sink,
                        encryptor,
                        self.amt,
                    )));
                }
            } else {
                if self.bytes == None {
                    match stream.poll() {
                        Ok(async_value) => {
                            match async_value {
                                Async::Ready(bytes_option) => {
                                    match bytes_option {
                                        Some(bytes) => {
                                            self.amt += bytes.len() as u64;
                                            if self.amt > self.size_limit {
                                                return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "File size limited"));
                                            }
                                            encryptor.input(&bytes[..]);
                                            self.bytes = Some(bytes);
                                        },
                                        None => {
                                            self.read_done = true;
                                        }
                                    }
                                },
                                Async::NotReady => {
                                    return Ok(Async::NotReady);
                                }
                            }
                        },
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }
            }

            if let Some(_) = self.bytes {
                match sink.start_send(self.bytes.take().unwrap()) {
                    Ok(async_sink) => {
                        if let futures::AsyncSink::NotReady(bytes) = async_sink {
                            self.bytes = Some(bytes);
                            return Ok(Async::NotReady);
                        }
                    },
                    Err(e) => {
                        return Err(e);
                    }
                }
            }

        }

    }
}

#[macro_export]
macro_rules! boxed {
    ($value:expr) => (
        {
            Box::new($value) as Box<dyn Future<Item = HttpResponse, Error = actix_web::Error>>
        }
    );
}

#[macro_export]
macro_rules! future_wrap {
    ($value:expr) => (
        {
            boxed!(futures::future::ok::<_, actix_web::Error>($value))
        }
    );
}


const SIGN_FF_D8_FF: &'static [u8] = &[ 0xFF, 0xD8, 0xFF ]; //JPG
const SIGN_89_50_4E_47_0D_0A_1A_0A: &'static [u8] = &[ 0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A ]; //PNG
const SIGN_47_49_46_38: &'static [u8] = &[ 0x47, 0x49, 0x46, 0x38 ]; //GIF
const SIGN_42_4D: &'static [u8] = &[ 0x42, 0x4D ]; //BMP

/// Возвращвает content-type сооответсвующий сигнатуре изображения по первым байтам файла
#[allow(dead_code)]
#[inline]
pub fn get_content_type_by_signature(src: &[u8]) -> Option<&'static str> {
    if src.len() > 3 && src.starts_with(SIGN_FF_D8_FF) { //JPG
        if match src[3] {
            0xE8 | 0xE3 | 0xE2 | 0xE1 | 0xE0 => true,
            _ => false
        } {
            return Some("image/jpeg");
        }
    }
    if src.starts_with(SIGN_89_50_4E_47_0D_0A_1A_0A) {
        return Some("image/png");
    }
    if src.starts_with(SIGN_47_49_46_38) {
        return Some("image/gif");
    }
    if src.starts_with(SIGN_42_4D) {
        return Some("image/bmp");
    }
    None
}
