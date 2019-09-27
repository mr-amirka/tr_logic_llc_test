use opencv::{
    core::{self, Mat, Size},
    prelude::*,
    types::*,
};

/// Конвертирует объект изображения в массив байт
#[allow(dead_code)]
#[inline]
pub fn mat_to_data(src_image: &Mat, extension: &str) -> Option<VectorOfuchar> {
    let mut buf = VectorOfuchar::new();
    match opencv::imgcodecs::imencode(
        extension,
        src_image,
        &mut buf,
        &Vector::new()
    ) {
        Ok(_) => {
            if buf.len() > 0 {
                return Some(buf);
            }
        },
        Err(err) => {
            eprintln!("image_mat_to_data: {:?}", err);
        }
    }

    None
}

/// Конвертирует срез байт в объект изображения
#[allow(dead_code)]
#[inline]
pub fn data_to_mat(src: &[u8]) -> Option<Mat> {
    if let Ok(source_image) = Mat::new_rows_cols_with_data(
        1,
        src.len() as i32,
        core::CV_8UC1,
        unsafe {
            &mut *(src.as_ptr() as *mut libc::c_void)
        },
        opencv::core::Mat_AUTO_STEP
    ) {
        if let Ok(image) = opencv::imgcodecs::imdecode(&source_image, opencv::imgcodecs::IMREAD_COLOR) {
            return Some(image);
        }
    }

    None
}

/// Изменяет размер изображения
#[allow(dead_code)]
pub fn mat_resize(src_image: &Mat, width: i32, height: i32) -> Option<Mat> {
    let image_size = match src_image.size() {
        Ok(v) => v,
        _ => {
            return None;
        }
    };
    let Size {
        width: original_width,
        height: original_height,
    } = image_size;

    if original_width < 1 || original_height < 1 {
        return None;
    }

    let mut h = height * original_width / width;
    let mut w = original_width;

    if h > original_height {
      w = w * original_height / h;
      h = original_height;
    }
    if w > original_width {
      h = h * original_width / w;
      w = original_width;
    }

    let rect = opencv::core::Rect::new(
        (original_width - w) / 2,
        (original_height - h) / 2,
        w,
        h,
    );
    let rect_image = match Mat::roi(&src_image, rect) {
        Ok(v) => v,
        _ => {
            return None;
        }
    };

    let mut resized_image = match unsafe {
        Mat::new_rows_cols(0, 0, core::CV_8UC1)
    } {
        Ok(v) => v,
        _ => {
            return None;
        }
    };

    let flag = if width > original_width || height > original_height {
        opencv::imgproc::INTER_CUBIC
    } else {
        opencv::imgproc::INTER_AREA
    };

    match opencv::imgproc::resize(
        &rect_image,
        &mut resized_image,
        opencv::core::Size::new(width, height),
        0.0,
        0.0,
        flag,
    ) {
        Ok(()) => Some(resized_image),
        _ => None
    }
}

/// Проверяет инвалидность формата изображения
#[allow(dead_code)]
#[inline]
pub fn is_invalid_format(src: &[u8]) -> bool {
    let image = match data_to_mat(src) {
        Some(image) => image,
        _ => {
            return true;
        }
    };

    match image.size() {
        Ok(size) => size.width < 1 || size.height < 1,
        _ => true
    }
}

/// Получает строку с именем поддерживаемого расширения для соответствующего mime типа
#[allow(dead_code)]
#[inline]
pub fn get_supported_extension_by_mime_name(mime_name: mime::Name) -> Option<&'static str> {
    match mime_name {
        mime::BMP => Some(".bmp"),
        mime::GIF => Some(".gif"),
        mime::JPEG => Some(".jpg"),
        mime::PNG => Some(".png"),
        _ => None
    }
}


const SIGN_FF_D8_FF: &'static [u8] = &[ 0xFF, 0xD8, 0xFF ]; //JPG
const SIGN_89_50_4E_47_0D_0A_1A_0A: &'static [u8] = &[ 0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A ]; //PNG
const SIGN_42_4D: &'static [u8] = &[ 0x42, 0x4D ]; //BMP

/// Возвращвает content-type сооответсвующий сигнатуре изображения по первым 8-ми байтам файла
#[allow(dead_code)]
#[inline]
pub fn get_content_type_by_signature(src: &[u8]) -> Option<&'static str> {
    if src.starts_with(SIGN_FF_D8_FF) { //JPG
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
    if src.starts_with(SIGN_42_4D) {
        return Some("image/bmp");
    }
    None
}
