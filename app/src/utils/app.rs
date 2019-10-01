use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use std::str::FromStr;
use crypto;
use crypto::digest::Digest;
use futures::{self, Future, Stream, Async};
use futures_cpupool::CpuPool;
use base64;
use bytes::Bytes;
use serde_json::json;
use positioned_io::WriteAt;
use actix_web::{
    client,
    HttpMessage,
    HttpResponse,
};
use utils::{
    self as utils,
    image,
    ImageInput,
    ImageOutputItem,
    ImageError,
};

pub static mut TEMPORARY_DIR: Option<std::path::PathBuf> = None;
pub static mut IMAGES_DIR: Option<std::path::PathBuf> = None;
pub static mut THUMBNAILS_DIR: Option<std::path::PathBuf> = None;
pub static mut FILE_MAX_SIZE: u64 = 10 * 1024 * 1024;
pub static mut JSON_MAX_SIZE: u64 = 10 * 1024 * 1024;
pub static mut TEXT_MAX_SIZE: u64 = 10 * 1024 * 1024;
pub static mut THUMBNAILS_RESOLUTION: (u32, u32) = (100, 100);

lazy_static! {
    static ref THREAD_COUNT: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));
}

thread_local! {
    pub static THREAD_ID: RefCell<u64> = {
        let mut count = THREAD_COUNT.lock().unwrap();
        *count += 1;
        RefCell::new(*count)
    };
    pub static THREAD_LOCAL_INDEX: RefCell<u64> = RefCell::new(0);
}

/// получить уникальный индекс счетчика текущего потока
#[allow(dead_code)]
#[inline]
pub fn get_thread_local_unique_index() -> u64 {
    let mut index = 0;
    THREAD_LOCAL_INDEX.with(|i| {
        *i.borrow_mut() += 1;
        index = *i.borrow();
    });
    index
}

/// получить индекс текущего потока
#[allow(dead_code)]
#[inline]
pub fn get_thread_id() -> u64 {
    let mut index = 0;
    THREAD_ID.with(|i| {
        *i.borrow_mut() += 1;
        index = *i.borrow();
    });
    index
}

macro_rules! get_dir {
    ($name:ident, $val:expr) => (
        unsafe {
            if let Some(val) = $name.as_ref() {
                val.as_path()
            } else {
                $name = Some(std::path::PathBuf::from_str($val).unwrap());
                $name.as_ref().unwrap().as_path()
            }
        }
    );
}

/// получить путь к директории временных файлов
#[allow(dead_code)]
#[inline]
pub fn get_tmp_dir() -> &'static std::path::Path {
    get_dir!(TEMPORARY_DIR, "./data/tmp/")
}

/// получить путь к директории изображений
#[allow(dead_code)]
#[inline]
pub fn get_images_dir() -> &'static std::path::Path {
    get_dir!(IMAGES_DIR, "./data/images/")
}

/// получить путь к директории миниатюр
#[allow(dead_code)]
#[inline]
pub fn get_thumbnails_dir() -> &'static std::path::Path {
    get_dir!(THUMBNAILS_DIR, "./data/thumbnails/")
}

/// получить резолюцию миниатюр
#[allow(dead_code)]
#[inline]
pub fn get_thumbnails_resolution() -> (u32, u32) {
    unsafe {
        THUMBNAILS_RESOLUTION
    }
}

#[allow(dead_code)]
#[inline]
pub fn get_file_max_size() -> u64 {
    unsafe { FILE_MAX_SIZE }
}

#[allow(dead_code)]
#[inline]
pub fn get_json_max_size() -> u64 {
    unsafe { JSON_MAX_SIZE }
}

#[allow(dead_code)]
#[inline]
pub fn get_text_max_size() -> u64 {
    unsafe { TEXT_MAX_SIZE }
}

/// Потокобезопасная генерация имени временного файла.
/// Мне известно, что в имлементации rocket-multipart-form-data 0.4.1 имена временных файлов
/// генерируются на основе текущего времени системы в наносекундах.
/// Однако, я не уверен, что такой подход гарантирует исключение коллизий, и в моем варианте
/// для надежности за основу взят индекс текущего потока и его локальный индексный счетчик.
#[allow(dead_code)]
#[inline]
pub fn get_tmp_file_name() -> std::path::PathBuf {
    get_tmp_dir().join(&format!("tmp-{}-{}", get_thread_id(), get_thread_local_unique_index()))
}

/// Загружает изображение
#[allow(dead_code)]
pub fn upload_image<Stream> (cpu_pool: &'static CpuPool, stream: Stream, extension: &'static str, size_limit: u64)
    -> impl futures::Future<Item = (std::path::PathBuf, u64), Error = std::io::Error>
where
    Stream: futures::stream::Stream<Item = bytes::Bytes, Error = std::io::Error>
{
    stream.fold((Vec::<Bytes>::new(), 0 as u64, true), move |(mut chunks, mut total, mut invalidated), chunk| {
        total += chunk.len() as u64;
        if total > size_limit {
            return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Size limited"));
        }

        // первичная проверка сигнатуры, как только будет получено достаточно данных
        if invalidated {
            if total > 7 {
                if let None = image::get_content_type_by_signature(&chunk[..]) {
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Signature Error"));
                } else {
                    invalidated = false;
                }
            }
        }

        chunks.push(chunk);
        Ok((chunks, total, invalidated))
    })
    .and_then(move |(src_chunks, total, invalidated)| {
        macro_rules! skip {
            () => (
                {
                    Box::new(
                        futures::future::err(
                            std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid Data")
                        )
                    ) as Box<dyn Future<Item = (std::path::PathBuf, u64), Error = std::io::Error>>
                }
            );
        }

        if invalidated {
            return skip!();
        }

        let mut encryptor = crypto::sha2::Sha256::new();
        let buf = {
            let chunks = src_chunks;
            let mut buf: Vec<u8> = Vec::with_capacity(total as usize);
            for chunk in chunks {
                let slice = &chunk[..];
                encryptor.input(slice);
                buf.extend_from_slice(slice);
            }
            buf
        };

        if let Some(original_image) = image::data_to_mat(&buf[..]) {
            let (w, h) = get_thumbnails_resolution();
            if let Some(thumbnail_image) = image::mat_resize(&original_image, w as i32, h as i32) {
                if let Some(chars_vec) = image::mat_to_data(&thumbnail_image, extension) {
                    let (src_path, src_subdir) = utils::get_file_path_by_hex(&encryptor.result_str(), extension);
                    return Box::new(
                        cpu_pool.spawn_fn(move || {
                            macro_rules! create_file {
                                ($filename:expr, $subdir:expr) => (
                                    {
                                        let filename = $filename;
                                        match
                                            std::fs::OpenOptions::new()
                                                .write(true)
                                                .create(true)
                                                .open(&filename)
                                        {
                                            Ok(file) => file,
                                            Err(err) => {
                                                match err.kind() {
                                                    std::io::ErrorKind::NotFound => {
                                                        std::fs::create_dir_all($subdir)?;
                                                        std::fs::OpenOptions::new()
                                                            .write(true)
                                                            .create(true)
                                                            .open(&filename)?
                                                        //std::fs::File::create(&filename)?
                                                    },
                                                    _ => {
                                                        return Err(err);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                );
                            }

                            let size = buf.len() as u64;
                            {
                                let local_buf = buf;
                                let images_root = get_images_dir();
                                let mut file = create_file!(images_root.join(&src_path), images_root.join(&src_subdir));
                                file.write_all_at(0, &local_buf[..])?;
                            }
                            {
                                let thumbnails_root = get_thumbnails_dir();
                                let mut file = create_file!(thumbnails_root.join(&src_path), thumbnails_root.join(&src_subdir));
                                // полностью скопировал исходники rust_opencv, чтоб сделать метод 'to_slice' публичным
                                file.write_all_at(0, chars_vec.to_slice())?;
                            }
                            Ok::<(std::path::PathBuf, _), _>((src_path.into(), size))
                        })
                    ) as Box<dyn Future<Item = (std::path::PathBuf, u64), Error = std::io::Error>>;
                }
            }
        }

        skip!()
    })

}

/// Предыдущая вариант функции 'upload_image' со стримовой загрузкой файла.
/// Стримово записывает данные во врменный файл, а затем по завершению приема данных переименовывет его в соответсвии с хэш суммой.
/// Изначально не подразумевала генерацию миниатюр, но так, как для генерации миниатюр
/// нужно держать данные исходного изображения в оперативной памяти,
/// то не имеет особого смысла стримить исходный файл сразу на диск.
/// Также при генерации миниатюр имеется возможность проверить валидность формата данных изображения,
/// поэтому более целесообразно записывать файл на диск только после проверки валидности целиком всего изображения.
#[allow(dead_code)]
pub fn upload_file<Stream> (cpu_pool: &'static CpuPool, stream: Stream, extension: &'static str, file_size_limit: u64)
    -> impl futures::Future<Item = (std::path::PathBuf, u64), Error = std::io::Error>
where
    Stream: futures::stream::Stream<Item = bytes::Bytes, Error = std::io::Error>
{
    let tmp_filename = get_tmp_file_name();
    let from_name = tmp_filename.clone();
    let new_tmp_filename = tmp_filename.clone();
    cpu_pool
        .spawn_fn(|| {
            let r = std::fs::File::create(&new_tmp_filename);
            if let Err(err) = r {
                return match err.kind() {
                    std::io::ErrorKind::NotFound => {
                        std::fs::create_dir_all(get_tmp_dir())?;
                        std::fs::File::create(new_tmp_filename)
                    },
                    _ => Err(err)
                };
            }
            r
        })
         .and_then(move |file| {
             utils::stream_to_sink(
                 stream,
                 async_fs::AsyncFileSink::from_std(cpu_pool, file),
                 //crypto::sha1::Sha1::new(),
                 //crypto::md5::Md5::new(),
                 crypto::sha2::Sha256::new(),
                 file_size_limit,
             )
                .and_then(move |(_stream, _file, mut encryptor, size)| {
                    let (src_path, src_subdir) = utils::get_file_path_by_hex(&encryptor.result_str(), extension);
                    let images_dir = get_images_dir();
                    let target_full_path = images_dir.join(&src_path);
                    cpu_pool.spawn_fn(move || {
                        if let Err(err) = std::fs::rename(&from_name, &target_full_path) {
                            match err.kind() {
                                std::io::ErrorKind::NotFound => {
                                    std::fs::create_dir_all(&images_dir.join(src_subdir))?;
                                    std::fs::rename(&from_name, &target_full_path)?;
                                },
                                _ => {
                                    return Err(err);
                                }
                            }
                        }
                        Ok((src_path.into(), size))
                    })
                })
         })
         .then(move |r| {
             let _ = cpu_pool.spawn_fn(|| std::fs::remove_file(tmp_filename));
             r
         })
}

/// Загружает изображение
#[allow(dead_code)]
pub fn image_upload(item: ImageInput) -> Box<dyn Future<Item = ImageOutputItem, Error = ImageError>> {
    macro_rules! skip {
        ($err:expr) => (
            {
                Box::new(
                    futures::future::err::<ImageOutputItem, ImageError>($err)
                ) as Box<dyn Future<Item = ImageOutputItem, Error = ImageError>>
            }
        );
    }

    let ImageInput {
        name: original_name_option,
        r#type: mime_type,
        content
    } = item;


    if let Some(extension) = image::get_supported_extension_by_mime_name(mime_type.subtype()) {
        let body_result = base64::decode(content.as_ref());
        if let Err(err) = body_result {
            eprintln!("IO error: {:?}", err);
            return skip!(ImageError::Invalid);
        }

        return Box::new(
            upload_image(
                &async_fs::DEFAULT_CPU_POOL,
                futures::stream::once::<_, std::io::Error>(
                    Ok(Bytes::from(body_result.unwrap()))
                ),
                extension,
                get_file_max_size(),
            )
            .then(|r| {
                match r {
                    Ok((name, size)) => Ok(ImageOutputItem {
                        original: match original_name_option {
                            Some(v) => Some(v.into()),
                            _ => None,
                        },
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
                }
            })
        ) as Box<dyn Future<Item = ImageOutputItem, Error = ImageError>>
    }
    skip!(ImageError::Unsupported)
}

/// Загружает изображение по URL
#[allow(dead_code)]
pub fn upload_url_image(url: &str) -> impl futures::Future<Item = (std::path::PathBuf, u64), Error = ImageError> {
    macro_rules! skip {
        ($err:expr) => (
            {
                Box::new(
                    futures::future::err($err)
                ) as Box<dyn Future<Item = (std::path::PathBuf, u64), Error = ImageError>>
            }
        );
    }

    client::Client::default()
        .get(url)
        //.header("User-Agent", "tr_logic_llc_test")
        .send()
        .then(move |r| {

            let response = match r {
                Ok(response) => response,
                Err(_err) => {
                    //eprintln!("Error: {:?}", err);
                    return skip!(ImageError::NetError);
                }
            };

            if !response.status().is_success() {
                return skip!(ImageError::Invalid);
            }

            let mime_type = match response.mime_type() {
               Ok(mime_type_option) => {
                   match mime_type_option {
                       Some(mime_type) => mime_type,
                       None => {
                           return skip!(ImageError::Invalid);
                       }
                   }
               },
               Err(_err) => {
                   //eprintln!("Error: {:?}", err);
                   return skip!(ImageError::Invalid);
               }
           };

           let max_size = match utils::get_content_length(&response.headers(), get_file_max_size()) {
               Ok(size) => size,
               _ => {
                   return skip!(ImageError::Overflow);
               }
           };

           if let Some(extension) = image::get_supported_extension_by_mime_name(mime_type.subtype()) {
               return Box::new(
                   upload_image(
                       &async_fs::DEFAULT_CPU_POOL,
                       response.map_err(|_err| {
                           //eprintln!("Error: {:?}", err);
                           std::io::Error::new(std::io::ErrorKind::Other, "Payload error")
                       }),
                       extension,
                       max_size,
                   )
                   .map_err(|err| {
                       if err.kind() == std::io::ErrorKind::UnexpectedEof {
                           ImageError::Overflow
                       } else {
                           ImageError::WriteError
                       }
                   })
               ) as Box<dyn Future<Item = (std::path::PathBuf, u64), Error = ImageError>>;
           }

           eprintln!("This mime type is not supported: {:?}", mime_type);
           skip!(ImageError::Unsupported)
        })
}

/// Загружает изображение по URL, в том числе со встроенным содержимым
#[allow(dead_code)]
#[inline]
pub fn upload_url_data_image(url: String) -> Box<dyn Future<Item = ImageOutputItem, Error = ImageError>> {
    if utils::is_url_data(&url) {
        match utils::data_url_parse_inner(url) {
            Ok(item) => image_upload(item),
            Err(err) => {
                Box::new(futures::future::err(
                    err
                )) as Box<dyn Future<Item = ImageOutputItem, Error = ImageError>>
            },
        }
    } else {
        Box::new(
            upload_url_image(&url).map(|(name, size)| ImageOutputItem {
                name,
                size,
                original: Some(url.into()),
            })
        ) as Box<dyn Future<Item = ImageOutputItem, Error = ImageError>>
    }
}

/// Загружает изображения путем конвертации массива байт из стрима, которые имеют формат URL строк,
/// в том числе со встроенным содержимым, разделенных символом перевода строки
pub fn upload_url_data_image_by_lines_stream<Stream> (stream: Stream, max_size: usize)
    -> impl futures::Future<Item = Vec<Result<ImageOutputItem, ImageError>>, Error = std::io::Error>
where
    Stream: futures::stream::Stream<Item = bytes::Bytes, Error = std::io::Error>
{
    macro_rules! bxd {
        ($value:expr) => (
            {
                Box::new($value) as Box<dyn Future<Item = Result<ImageOutputItem, ImageError>, Error = std::io::Error>>
            }
        );
    }
    macro_rules! wrap_url {
        ($url:expr) => (
            {
                match String::from_utf8($url) {
                    Ok(url) => bxd!(upload_url_data_image(url).then(|r| Ok(r))),
                    _ => bxd!(futures::future::ok(Err(ImageError::Invalid))),
                }
            }
        );
    }

    stream
        .fold((
            Vec::<Box<dyn Future<Item = Result<ImageOutputItem, ImageError>, Error = std::io::Error>>>::new(),
            Vec::new(),
            0 as usize
        ), move |(mut urls, mut last, mut size), item| {
            let mut offset_ptr = item.as_ptr() as usize;
            let end_ptr = offset_ptr + item.len();

            #[allow(unused_assignments)]
            let mut line_length: usize = 0;
            let mut line_start: usize = offset_ptr;

            macro_rules! next {
                () => (
                    {
                        line_length = offset_ptr - line_start;
                        size += line_length;
                        if size > max_size {
                            return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Overflow"));
                        }
                        last.extend_from_slice(unsafe {
                            std::slice::from_raw_parts(line_start as *const u8, line_length)
                        });
                    }
                );
            }

            'all: while offset_ptr < end_ptr {
                'string_line: while offset_ptr < end_ptr {
                    match unsafe { *(offset_ptr as *const u8) } {
                        b'\r' | b'\n' | b' ' | b'\t' => {
                            next!();

                            urls.push(wrap_url!(last));
                            last = Vec::new();

                            offset_ptr += 1;
                            'space: while offset_ptr < end_ptr {
                                match unsafe { *(offset_ptr as *const u8) } {
                                    b'\r' | b'\n' | b' ' | b'\t' => {
                                        offset_ptr += 1;
                                    },
                                    _ => {
                                        break;
                                    }
                                }
                            }
                            line_start = offset_ptr;
                            break;
                        },
                        _ => {
                            offset_ptr += 1;
                        }
                    }
                }

            }
            next!();

            Ok((urls, last, size))
        })
        .and_then(|(mut urls, last, _)| {
            if last.len() > 0 {
                urls.push(wrap_url!(last));
            }
            futures::future::join_all(urls)
        })
}

#[allow(dead_code)]
#[inline]
pub fn open_file_to_response(path: std::path::PathBuf) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    async_fs::DEFAULT_CPU_POOL.spawn_fn(|| {
        let file = std::fs::File::open(path)?;
        let metadata = file.metadata()?;
        if metadata.is_file() {
            Ok(file)
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Not found"))
        }
    })
        .and_then(|file| {
            let mut source_stream_option = Some(async_fs::AsyncFileStream::from_std(
                &async_fs::DEFAULT_CPU_POOL,
                file,
                8192
            ));
            futures::future::poll_fn(move || {
                let mut chunk_option = if let Async::Ready(chunk_option) = source_stream_option.as_mut().unwrap().poll()? {
                    chunk_option
                } else {
                    return Ok(Async::NotReady);
                };

                let content_type = if let Some(chunk) = &chunk_option {
                    if let Some(content_type) = utils::get_content_type_by_signature(&chunk[..]) {
                        content_type
                    } else {
                        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Signature Error"));
                    }
                } else {
                    return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Unexpected Eof"));
                };

                let mut source_stream = source_stream_option.take().unwrap();
                let stream = futures::stream::poll_fn(move || {
                    if let Some(chunk) = chunk_option.take() {
                        return Ok(Async::Ready(Some(chunk)));
                    }
                    source_stream.poll()
                });

                Ok(Async::Ready((content_type, Arc::new(Mutex::new(Some(stream))))))

            })
        })
        .then(|r| {
            match r {
                Ok((content_type, stream_option_arc)) => Ok(
                    HttpResponse::Ok()
                        .header(actix_web::http::header::CONTENT_TYPE, content_type)
                        .streaming(stream_option_arc.lock().unwrap().take().unwrap())
                ),
                Err(err) => {
                    eprintln!("IO error: {:?}", err);
                    Err(actix_web::Error::from(HttpResponse::NotFound().json(
                        json!({
                            "Err": {
                                "type": "NotFound"
                            }
                        })
                    )))
                }
            }
        })
}
