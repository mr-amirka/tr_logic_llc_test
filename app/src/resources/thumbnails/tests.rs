#[cfg(test)]

extern crate test;

#[test]
pub fn it_get_one() {
    use futures::{Future};
    use actix_web;
    use utils::{app};
    use resources;

    let address = "0.0.0.0:8200";

    let thumbnails_dir = app::get_thumbnails_dir();
    let _ = std::fs::create_dir_all(&thumbnails_dir);
    let target_path = thumbnails_dir.join("green_0006.jpg");
    std::fs::copy("./assets/green_0006.jpg", &target_path).unwrap();

    let (tx, rx) = std::sync::mpsc::channel();
    let _ = std::thread::spawn(move || {
        let sys = actix_rt::System::new("thumbnails_get_one-server");

        let addr = actix_web::HttpServer::new(|| {
            actix_web::App::new()
                .route(
                    "/thumbnails/{filename:.*}",
                    actix_web::web::get().to_async(resources::thumbnails::get_one::route)
                )
        })
            .bind(address)
            .unwrap()
            .start();

        let _ = tx.send(addr);
        let _ = sys.run();

    });
    let addr = rx.recv().unwrap();

    let _ = actix_rt::System::new("thumbnails_get_one").block_on(futures::lazy(|| {
        actix_web::client::Client::default()
            .get(format!("http://{}/thumbnails/green_0006.jpg", address))
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

                        assert_eq!(body_bytes.len(), 98236);

                        std::fs::remove_file(target_path).unwrap();

                        let _ = addr
                            .stop(true)
                            .wait();

                        Ok::<(),()>(())
                    })
                ) as Box<dyn Future<Item = (), Error = ()>>;
            })
    }));
}
