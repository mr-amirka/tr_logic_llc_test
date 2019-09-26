#[cfg(test)]

extern crate test;

use bytes::Bytes;
use futures_cpupool::{CpuPool};

lazy_static! {
    static ref TEST_IMAGE_DATA: Bytes = Bytes::from(base64::decode(
        "iVBORw0KGgoAAAANSUhEUgAAABgAAAAYCAMAAADXqc3KAAAB+FBMVEUAAAA/mUPidDHiLi5Cn0XkNTPmeUrkdUg/m0Q0pEfcpSbwaVdKskg+lUP4zA/iLi3msSHkOjVAmETdJSjtYFE/lkPnRj3sWUs8kkLeqCVIq0fxvhXqUkbVmSjwa1n1yBLepyX1xxP0xRXqUkboST9KukpHpUbuvRrzrhF/ljbwaljuZFM4jELaoSdLtElJrUj1xxP6zwzfqSU4i0HYnydMtUlIqUfywxb60AxZqEXaoifgMCXptR9MtklHpEY2iUHWnSjvvRr70QujkC+pUC/90glMuEnlOjVMt0j70QriLS1LtEnnRj3qUUXfIidOjsxAhcZFo0bjNDH0xxNLr0dIrUdmntVTkMoyfL8jcLBRuErhJyrgKyb4zA/5zg3tYFBBmUTmQTnhMinruBzvvhnxwxZ/st+Ktt5zp9hqota2vtK6y9FemNBblc9HiMiTtMbFtsM6gcPV2r6dwroseLrMrbQrdLGdyKoobKbo3Zh+ynrgVllZulTsXE3rV0pIqUf42UVUo0JyjEHoS0HmsiHRGR/lmRz/1hjqnxjvpRWfwtOhusaz0LRGf7FEfbDVmqHXlJeW0pbXq5bec3fX0nTnzmuJuWvhoFFhm0FtrziBsjaAaDCYWC+uSi6jQS3FsSfLJiTirCOkuCG1KiG+wSC+GBvgyhTszQ64Z77KAAAARXRSTlMAIQRDLyUgCwsE6ebm5ubg2dLR0byXl4FDQzU1NDEuLSUgC+vr6urq6ubb29vb2tra2tG8vLu7u7uXl5eXgYGBgYGBLiUALabIAAABsElEQVQoz12S9VPjQBxHt8VaOA6HE+AOzv1wd7pJk5I2adpCC7RUcHd3d3fXf5PvLkxheD++z+yb7GSRlwD/+Hj/APQCZWxM5M+goF+RMbHK594v+tPoiN1uHxkt+xzt9+R9wnRTZZQpXQ0T5uP1IQxToyOAZiQu5HEpjeA4SWIoksRxNiGC1tRZJ4LNxgHgnU5nJZBDvuDdl8lzQRBsQ+s9PZt7s7Pz8wsL39/DkIfZ4xlB2Gqsq62ta9oxVlVrNZpihFRpGO9fzQw1ms0NDWZz07iGkJmIFH8xxkc3a/WWlubmFkv9AB2SEpDvKxbjidN2faseaNV3zoHXvv7wMODJdkOHAegweAfFPx4G67KluxzottCU9n8CUqXzcIQdXOytAHqXxomvykhEKN9EFutG22p//0rbNvHVxiJywa8yS2KDfV1dfbu31H8jF1RHiTKtWYeHxUvq3bn0pyjCRaiRU6aDO+gb3aEfEeVNsDgm8zzLy9egPa7Qt8TSJdwhjplk06HH43ZNJ3s91KKCHQ5x4sw1fRGYDZ0n1L4FKb9/BP5JLYxToheoFCVxz57PPS8UhhEpLBVeAAAAAElFTkSuQmCC"
    ).unwrap());
    static ref TEST_CPU_POOL: CpuPool = CpuPool::new(1);
}


#[test]
fn it_upload_image() {
    use std::str::FromStr;
    use futures::Future;
    use std::io::Read;
    use utils::{app};

    let stream = futures::stream::once::<_, std::io::Error>(
        Ok(TEST_IMAGE_DATA.clone())
    );
    let extension = "_upload_image.png";
    let size_limit: u64 = 2048;

    let future = app::upload_image(
        &TEST_CPU_POOL,
        stream,
        extension,
        size_limit
    );
    let (file_path, file_size) = future.wait().unwrap();

    assert_eq!(file_path, std::path::PathBuf::from_str("Z/W/j/J6cNaf6BvI26J9wTYybRxg6JPLJeNumyS4nR-OhM_upload_image.png").unwrap());
    assert_eq!(file_size, 1086);

    let image_file_path = app::get_images_dir().join(&file_path);

    let data = {
        let mut data: Vec<u8> = Vec::new();

        std::fs::File::open(&image_file_path)
            .unwrap()
            .read_to_end(&mut data)
            .unwrap();
        let _ = std::fs::remove_file(&image_file_path).unwrap();

        data
    };

    assert_eq!(&data[..], &TEST_IMAGE_DATA[..]);

    let thumbnail_file_path = app::get_thumbnails_dir().join(&file_path);
    let has_data = {
        let mut data: Vec<u8> = Vec::new();

        std::fs::File::open(&thumbnail_file_path)
            .unwrap()
            .read_to_end(&mut data)
            .unwrap();
        let _ = std::fs::remove_file(&thumbnail_file_path);

        data.len() > 0
    };

    assert_eq!(has_data, true);
}


#[test]
fn it_upload_file() {
    use std::str::FromStr;
    use futures::Future;
    use std::io::Read;
    use utils::{app};

    let stream = futures::stream::once::<_, std::io::Error>(
        Ok(TEST_IMAGE_DATA.clone())
    );
    let extension = "_upload_file.png";
    let size_limit: u64 = 2048;

    let future = app::upload_file(
        &TEST_CPU_POOL,
        stream,
        extension,
        size_limit
    );
    let (file_path, file_size) = future.wait().unwrap();

    assert_eq!(file_path, std::path::PathBuf::from_str("Z/W/j/J6cNaf6BvI26J9wTYybRxg6JPLJeNumyS4nR-OhM_upload_file.png").unwrap());
    assert_eq!(file_size, 1086);

    let image_file_path = app::get_images_dir().join(&file_path);

    let data = {
        let mut data: Vec<u8> = Vec::new();

        std::fs::File::open(&image_file_path)
            .unwrap()
            .read_to_end(&mut data)
            .unwrap();
        let _ = std::fs::remove_file(&image_file_path).unwrap();

        data
    };

    assert_eq!(&data[..], &TEST_IMAGE_DATA[..]);
}

#[test]
fn it_image_upload() {
    use std::str::FromStr;
    use futures::Future;
    //use std::io::Read;
    use utils::{app, ImageInput, ImageOutputItem};

    let content = "iVBORw0KGgoAAAANSUhEUgAAABgAAAAYCAYAAADgdz34AAAFCUlEQVR42q1WWUieRxSdcd/3XYMLaqgrWk1cSLG2RtGK4lN9ifRJ0ZdSBLGP+iAF+2CUPknFBfrQEFGMVFwgikmKS5pSRMQd477HfZueM/EvfqbpUweGmX++77vn3nPPvfNL8R9jcXHRztzc3NnS0tLZwsLC1szMzJznV1dXlxcXF8fn5+c7WPcCAwNPPmZD3j4oKChwg4HEw8PDtODg4Ch3d3dfHx8fT1dXVxtra2sNcHp6erm9vX2ysbGxvrW1tTw7O/unvb39c5yPdnd3734UoLCw8Mfo6OivPDw8vO/cuWPn6OhogQ+lra2tgPd6np2d6Qlj4ujoSJycnCisFwA7BOjq+Ph4+5MnT77/AKCoqOiX6urqr+G58PT0FDAsQI0ARaZXFN+HYWUCwZTHx8d6z/Xdu3ciICBAVFVVtTQ3NxcZAIqLi1/hwX3QozD1MwwF7uk53+O5BOcK3AtOgEkY14BKKUaqACBrampe19bWxhsAsrOzh9PT01NAkQgJCRHgnB/pSUAa5AoAvec5HNARIjc62v39fQpDwPuRtra2ewaA/Pz8l5GRkUn02snJSfj6+kpvb2+FPAgHBwftHabEXpG+a+p0ZMwHEi1XVlYUKJbt7e3jTU1NnxoAcnJyeuLj4x/a2dlpowCR3FtZWdFDDeDi4iKhKsXobGxs3nOIwbwtLy/Lzc1NwYkIep8+ffrQABAXF+cXExPzIjQ0NBCGBKRJz3X4BKFBRkYB8PntMT09LaampsSzZ8+mhoeH0968ebNiAMjIyPiupKTkh4mJCQtITsF7TQeNIiIdAaPCXpFv5gGqkWtra2ppaUnMzc1JLy8vERsbe97Y2Phtf3//TwaA8vLyVxUVFffHxsb0B+BTUuM0dHl5KTG1TKkiqsn0G7lQjA75kohaZWVlyYaGhnFI3piD3Nzc4UePHqVERUVp3lGhcm9vj0VEdUgAKQIRwKQgvKfzRMr8/PwkIlQoNllXVzfa0tKSaADIy8sbDg8PT6E3qGKBXAhUtObeVHBcb+75jHsCzs/Pi4WFBQEA0dXVNQYVJdyO4GViYmISPqQchYlv7mFI5wDqkW5ubtpjJt8k04ODA+pfImqFipaogdHW1lZjBGlpaYVobj+HhYXZwIhWCvVOuugtgShfRkWZ8vzmePv2rY6ir6/vcGBg4JvBwcFfDQCpqamRqOQetAV/6Fqx0JydnbWK2PDINX5LGKfCdA6QH7mzs6NWV1fFyMiIZDNEvuYg0y8xZw0AZWVlA5WVlZ8DWa2vrwt8qBN7nVB+rNiTuCLZ//QiJp1gpBQyVegIEn1o6PHjx58ZAJKTk39HYu6RFqiHlSnILVXESmUfIhh7EfcEYSNkLhiRv7+/nqQPanzd09NjbHZI8AgoSgA16u7duyIoKEgiudo7vscuK98P6l/fDZQp3yEInJEzMzOMXtbX1//R29sbZwB48ODBX5mZmZE0RI+QTIlEK7YLVjUNkQaqiGdMvElFjJKVzKpmEQJgsrOz8xMDABTUX1pamk5t0yNSReVwz2nqRVQRVXYtUz14HzBvvHDYBWBnAL3piw+uTHTKpqSkpNSIiAgvFJ0tjJnDWzO2aYJRujTOCK5VxFZ9BeVcDg0NHaP/rHZ0dDzH7+J/vZNJLS59l8nJyYTd3d0sAPjAa1fkxQOrvqPhOVV0AW+PIIRNFNcO1lVE+BsuqpEbl74Z5tUH/yr+7/E3QavLN5bb1BAAAAAASUVORK5CYII=";

    let image_input_item = ImageInput {
        name: Some("filename.png".into()),
        r#type: mime::IMAGE_PNG,
        content: Bytes::from(content)
    };

    let ImageOutputItem {
        original,
        name,
        size
    } = app::image_upload(image_input_item)
        .wait()
        .unwrap();

    let _ = std::fs::remove_file(app::get_images_dir().join(&name)).unwrap();
    let _ = std::fs::remove_file(app::get_thumbnails_dir().join(&name)).unwrap();

    assert_eq!(original, Some("filename.png".into()));
    assert_eq!(name, std::path::PathBuf::from_str("2/f/C/GCUNExKaKs9VN6GthiSiulxCjCgF0pWjJT0RGxxk.png").unwrap());
    assert_eq!(size, 1346);
}


#[test]
fn it_upload_url_image() {
    use std::str::FromStr;
    use futures::Future;
    use actix_files::NamedFile;
    use actix_web;
    use utils::{app};

    let address = "127.0.0.1:8088";

    fn index(req: actix_web::HttpRequest) -> actix_web::Result<NamedFile> {
        let path = std::path::PathBuf::from_str("./assets").unwrap()
            .join(
                req
                    .match_info()
                    .query("filename")
                    .parse::<std::path::PathBuf>()
                    .unwrap()
            );
        Ok(NamedFile::open(path)?)
    }

    let (tx, rx) = std::sync::mpsc::channel();
    let _ = std::thread::spawn(move || {
        let sys = actix_rt::System::new("upload_url_image-server");

        let addr = actix_web::HttpServer::new(|| {
            actix_web::App::new().route("/assets/{filename:.*}", actix_web::web::get().to(index))
        })
            .bind(address)
            .unwrap()
            .start();

        let _ = tx.send(addr);
        let _ = sys.run();

    });
    let addr = rx.recv().unwrap();

    let _ = actix_rt::System::new("upload_url_image").block_on(futures::lazy(|| {
        app::upload_url_image(
            &format!("http://{}/assets/image2.jpg", address)
        ).then(|r| {
            let (name, size) = r.unwrap();

            let _ = std::fs::remove_file(app::get_images_dir().join(&name)).unwrap();
            let _ = std::fs::remove_file(app::get_thumbnails_dir().join(&name)).unwrap();

            assert_eq!(name, std::path::PathBuf::from_str("X/F/O/pd0O-Zm3M-C40uB7VgDcDokB7v2glwmF4BmJTY0w.jpg").unwrap());
            assert_eq!(size, 166985);

            let _ = addr
                .stop(true)
                .wait();

            Ok::<_,()>(())
        })
    }));
}


#[test]
fn it_upload_url_data_image() {
    use std::str::FromStr;
    use futures::Future;
    use actix_files::NamedFile;
    use actix_web;
    use utils::{app, ImageOutputItem};

    let address = "127.0.0.1:8089";

    fn index(req: actix_web::HttpRequest) -> actix_web::Result<NamedFile> {
        let path = std::path::PathBuf::from_str("./assets").unwrap()
            .join(
                req
                    .match_info()
                    .query("filename")
                    .parse::<std::path::PathBuf>()
                    .unwrap()
            );
        Ok(NamedFile::open(path)?)
    }

    let (tx, rx) = std::sync::mpsc::channel();
    let _ = std::thread::spawn(move || {
        let sys = actix_rt::System::new("upload_url_data_image-server");

        let addr = actix_web::HttpServer::new(|| {
            actix_web::App::new().route("/assets/{filename:.*}", actix_web::web::get().to(index))
        })
            .bind(address)
            .unwrap()
            .start();

        let _ = tx.send(addr);
        let _ = sys.run();

    });
    let addr = rx.recv().unwrap();

    let image_url = format!("http://{}/assets/image.jpg", address);

    let _ = actix_rt::System::new("upload_url_data_image").block_on(futures::lazy(|| {
        app::upload_url_data_image(image_url.clone()).then(|r| {
            let ImageOutputItem {
                name,
                original,
                size,
            } = r.unwrap();

            let _ = std::fs::remove_file(app::get_images_dir().join(&name)).unwrap();
            let _ = std::fs::remove_file(app::get_thumbnails_dir().join(&name)).unwrap();

            assert_eq!(original, Some(image_url.into()));
            assert_eq!(name, std::path::PathBuf::from_str("l/k/C/mEWnboMIm0DlvU85EcF2ILPLlP6bLeTg3vps4C84.jpg").unwrap());
            assert_eq!(size, 555146);

            let _ = addr
                .stop(true)
                .wait();

            Ok::<_,()>(())
        })

    }));
}


#[test]
fn it_upload_url_data_image_with_data() {
    use std::str::FromStr;
    use futures::Future;
    use utils::{app, ImageOutputItem};

    let image_data_url = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABgAAAAYCAYAAADgdz34AAAD5UlEQVR42rVUS2tkRRT+6t7u2317OpNOzxgZHPBBwB8jzEIIIQlko2OPGSMGCQkJySJ0Q/ID0kNMgi4kMURmIYqKoOJKBBmQQVxk5cJHwDiTpO+rqm55Tt1O2+kImcBYUJxT9/F953zn1BGgNTs7K33fh5d34XkFFDwfvl9AqVxC3iuir1SCVyzCcRy4uRwE/WOUgpQSQRwjDAKEYYgobCFohfQsQisI0ajX86JNYN5+s4YELlyHdo6A8h5ybg55AnRcAZHLE7CAEQ6MSZEaA600tNZQKoGWGlJJKKlQdlOsrW+i3qiLDsHUndu4/6iKvDAELODnHfrQQTFHsPSVSg1CafA4UjhKtLUtuzWiWCFODKJE4TjQ+OCVMpobW6jXewg+PRrAs5C4Usih38+jROCMLpVBoBgsJRKFgIEJtEV+RH5C7xIijSmLn/5y8f6tq1jb2CSJGmcJvmxVcTNvUCXwK56AToFj+vEk1miR5QgjmRKQavsUOWWQsKXnbB8euvjw1QE039s6KxHX4Lv4Gl4sAn2eYyV5TFEeUcrHMoswUQSS0E4JMM6AkzYh2yRhAgcfjVzHvW6J5ufnzVu12/hBXsfzvoFDzxj8kMCDOItOKm13InV21spKkp2zzQV++KeL+xODJNEWd1FGsLCwYAke6Gu44ZHeFOVhIHFC+sYWlAkyUJZItuWIuzKQ1jf4+Q+BT167kRWZJTLGiMXFxfQu1eCBrKIqUhxR1I+IIGQZqA0lFcPKw63YjjqygNoC24y4yPQNE3x+57l/i9xN8H04gH6Q7hT5SaipYyQk1eI0Yqn+I/LkVJ6M4JffXXwxddNm0OiuwRRJ9HWLCAy3IN1Q6o5Qc+omS78NfqaotvgqC4BaVWkmcPDVOy+cJ7j7xuv47LiCik4QclTtlrSFtR2UScOApy3Zkcze5pS2shl88+5L59uU78HHf/fjGbr2/HNI0UiqRcK2rfupPFbzdlYsDVs7MjQV+VeBb+eGsEYEjV6Cjd+uggYMFVfZAkdRdrlivmSRsrLJRCJh4iQLQBEZzyFNgaQaCCPgx+WX0dykWdR9k//3Ydc9rv2ij2KBLI1rj8Z1+YJxHdG4jiIa08G5cZ0RXLTGx8cN2xIFUe7rQ6VSQYlIZ2ZmLvy/88HKyop5ErInXXNzc6JDsLy8bCYnJ+2Lg4MDDA4OXsrnxeduv9lsYmlpSQieQ8PDw1hdXUW7Hpf2efG519/b24OYnp42IyMjGBoasi/39/cv7fPic6+/u7sLUavVzOjoKNbX1+1LOl/a58XnXn9nZwdiYmLCjI2NPc36dtb29jYEyfNUu6d3/QNfVIZZQV5+3gAAAABJRU5ErkJggg==";

    let _ = actix_rt::System::new("upload_url_data_image_with_data").block_on(futures::lazy(|| {
        app::upload_url_data_image(image_data_url.into()).then(|r| {
            let ImageOutputItem {
                name,
                original,
                size,
            } = r.unwrap();

            let _ = std::fs::remove_file(app::get_images_dir().join(&name)).unwrap();
            let _ = std::fs::remove_file(app::get_thumbnails_dir().join(&name)).unwrap();

            assert_eq!(original, None);
            assert_eq!(name, std::path::PathBuf::from_str("v/q/A/H-TX4mR4L87IFNi60SrLQNRtilXX71fdzwd2yN4c.png").unwrap());
            assert_eq!(size, 1054);

            Ok::<_,()>(())
        })

    }));
}


#[test]
fn it_upload_url_data_image_by_lines_stream() {
    use std::str::FromStr;
    use futures::Future;
    use actix_files::NamedFile;
    use actix_web;
    use utils::{self, app, ImageOutputItem};

    fn index(req: actix_web::HttpRequest) -> actix_web::Result<NamedFile> {
        let path = std::path::PathBuf::from_str("./assets").unwrap()
            .join(
                req
                    .match_info()
                    .query("filename")
                    .parse::<std::path::PathBuf>()
                    .unwrap()
            );
        Ok(NamedFile::open(path)?)
    }

    let (tx, rx) = std::sync::mpsc::channel();
    let _ = std::thread::spawn(move || {
        let sys = actix_rt::System::new("upload_url_data_image_by_lines_stream-server");

        let addr = actix_web::HttpServer::new(|| {
            actix_web::App::new().route("/assets/{filename:.*}", actix_web::web::get().to(index))
        })
            .bind("127.0.0.1:8090")
            .unwrap()
            .start();

        let _ = tx.send(addr);
        let _ = sys.run();

    });
    let addr = rx.recv().unwrap();


    let image_urls = r##"http://127.0.0.1:8090/assets/closed_door.png
data:image/png;base64,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
http://127.0.0.1:8090/assets/for_error.png
http://127.0.0.1:8090/assets/cat.png"##;

    let stream = futures::stream::once::<_, std::io::Error>(
        Ok(Bytes::from(image_urls))
    );
    let max_size = app::get_text_max_size() as usize;

    let _ = actix_rt::System::new("upload_url_data_image_by_lines_stream").block_on(futures::lazy(|| {
        app::upload_url_data_image_by_lines_stream(
            stream,
            max_size,
        ).then(|r| {

            let mut items = r.unwrap();
            let four = items.pop().unwrap();
            let three = items.pop().unwrap();
            let two = items.pop().unwrap();
            let one = items.pop().unwrap();

            {
                let ImageOutputItem {
                    name,
                    original,
                    size,
                } = one.unwrap();

                let _ = std::fs::remove_file(app::get_images_dir().join(&name)).unwrap();
                let _ = std::fs::remove_file(app::get_thumbnails_dir().join(&name)).unwrap();

                assert_eq!(original, Some(std::path::PathBuf::from_str("http://127.0.0.1:8090/assets/closed_door.png").unwrap()));
                assert_eq!(name, std::path::PathBuf::from_str("S/X/i/QGKBbK9HM3AKWhsPnb0wEtyXma1xuZtYcrgfQCjk.png").unwrap());
                assert_eq!(size, 1117);
            }

            {
                let ImageOutputItem {
                    name,
                    original,
                    size,
                } = two.unwrap();

                let _ = std::fs::remove_file(app::get_images_dir().join(&name)).unwrap();
                let _ = std::fs::remove_file(app::get_thumbnails_dir().join(&name)).unwrap();

                assert_eq!(original, None);
                assert_eq!(name, std::path::PathBuf::from_str("i/l/v/viKCT7OkudCOdx9qlf7I3EyssFgILT712u-EFTWE.png").unwrap());
                assert_eq!(size, 1660);
            }

            assert_eq!(three, Err::<utils::ImageOutputItem, _>(utils::ImageError::Invalid));

            {
                let ImageOutputItem {
                    name,
                    original,
                    size,
                } = four.unwrap();

                let _ = std::fs::remove_file(app::get_images_dir().join(&name)).unwrap();
                let _ = std::fs::remove_file(app::get_thumbnails_dir().join(&name)).unwrap();

                assert_eq!(original, Some(std::path::PathBuf::from_str("http://127.0.0.1:8090/assets/cat.png").unwrap()));
                assert_eq!(name, std::path::PathBuf::from_str("S/U/b/dJ3CjbLcJxGJYxG7G5IOHoJGLOEGhmhd49mTFQyo.png").unwrap());
                assert_eq!(size, 1503);
            }

            let _ = addr
                .stop(true)
                .wait();

            Ok::<_,()>(())
        })
    }));
}


#[test]
fn it_open_file_to_response() {
    use std::str::FromStr;
    use futures::{Future};
    use actix_web::{
        self,
        HttpRequest,
        HttpResponse,
    };
    use utils::{app};

    let address = "127.0.0.1:8091";

    pub fn route(req: HttpRequest) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
        app::open_file_to_response(
            std::path::PathBuf::from_str("./assets").unwrap()
                .join(
                    req
                        .match_info()
                        .query("filename")
                        .parse::<std::path::PathBuf>()
                        .unwrap()
                )
        )
    }

    let (tx, rx) = std::sync::mpsc::channel();
    let _ = std::thread::spawn(move || {
        let sys = actix_rt::System::new("open_file_to_response-server");

        let addr = actix_web::HttpServer::new(|| {
            actix_web::App::new()
                .route("/assets/{filename:.*}", actix_web::web::get().to_async(route))
        })
            .bind(address)
            .unwrap()
            .start();

        let _ = tx.send(addr);
        let _ = sys.run();

    });
    let addr = rx.recv().unwrap();

    let _ = actix_rt::System::new("open_file_to_response").block_on(futures::lazy(|| {
        actix_web::client::Client::default()
            .get(format!("http://{}/assets/green_0004.jpg", address))
            .send()
            .then(move |r| {
                let mut response = r.unwrap();

                assert_eq!(response.status(), actix_web::http::StatusCode::OK);
                assert_eq!(
                    response.headers().get("content-type"),
                    Some(&actix_web::http::header::HeaderValue::from_static("image/jpeg"))
                );

                return Box::new(
                    response.body().then(move |r| {
                        let body_bytes = r.unwrap();

                        assert_eq!(body_bytes.len(), 79359);

                        let _ = addr
                            .stop(true)
                            .wait();

                        Ok::<(),()>(())
                    })
                ) as Box<dyn Future<Item = (), Error = ()>>;
            })
    }));
}
