#![feature(async_await)]
#![feature(test)]
#![feature(rustc_private)]

extern crate libc;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate futures;
extern crate async_fs;
extern crate opencv;
extern crate actix_web;
extern crate actix;
extern crate actix_rt;
extern crate num_cpus;
extern crate env_logger;
extern crate actix_files;
extern crate serde;
extern crate crypto;
extern crate serde_derive;
extern crate serde_json;
extern crate base64;
extern crate actix_multipart;
extern crate mime;
extern crate futures_cpupool;
extern crate bytes;
extern crate graceful;
extern crate config;
extern crate positioned_io;

use actix_web::{
    App,
    HttpServer,
    http,
    middleware,
};
use actix_web::middleware::Logger;
use actix_web::middleware::errhandlers::ErrorHandlers;

#[macro_use] mod utils;
mod routes;
mod resources;

use utils::app;
use graceful::SignalGuard;
use futures::future::Future;

fn main() {
    std::env::set_var("RUST_LOG", "my_errors=debug,actix_web=info");
    std::env::set_var("RUST_BACKTRACE", "1");

    env_logger::init();

    let signal_guard = SignalGuard::new();

    let settings = {
        let mut settings = config::Config::default();
        settings
            .merge(config::File::with_name("./data/config.toml")).unwrap()
            .merge(config::Environment::with_prefix("APP")).unwrap();

        settings.try_into::<std::collections::HashMap<String, String>>().unwrap()
    };

    //configs
    unsafe {
        app::TEMPORARY_DIR = Some(settings.get("TEMP_DIR").map_or("./data/tmp/".into(), |v| v.into()));
        app::IMAGES_DIR = Some(settings.get("IMAGES_DIR").map_or("./data/images/".into(), |v| v.into()));
        app::THUMBNAILS_DIR = Some(settings.get("THUMBNAILS_DIR").map_or("./data/thumbnails/".into(), |v| v.into()));

        if let Some(sv) = settings.get("FILE_MAX_SIZE") {
            if let Ok(v) = sv.parse::<u64>() {
                app::FILE_MAX_SIZE = v;
            }
        }
        if let Some(sv) = settings.get("JSON_MAX_SIZE") {
            if let Ok(v) = sv.parse::<u64>() {
                app::JSON_MAX_SIZE = v;
            }
        }
        if let Some(sv) = settings.get("TEXT_MAX_SIZE") {
            if let Ok(v) = sv.parse::<u64>() {
                app::TEXT_MAX_SIZE = v;
            }
        }

        let mut w = 100;
        let mut h = 100;

        if let Some(sv) = settings.get("THUMBNAILS_WIDTH") {
            if let Ok(v) = sv.parse::<u32>() {
                w = v;
            }
        }
        if let Some(sv) = settings.get("THUMBNAILS_HEIGHT") {
            if let Ok(v) = sv.parse::<u32>() {
                h = v;
            }
        }
        app::THUMBNAILS_RESOLUTION = (w, h);
    }

    let (tx, rx) = std::sync::mpsc::channel();
    let _ = std::thread::spawn(move || {
        println!("Server thread started. Type Ctrl+C to stop.");

        let sys = actix_rt::System::new("http-server");

        let addr = HttpServer::new(|| {
            App::new()
                .wrap(Logger::default())
                .wrap(Logger::new("%a %{User-Agent}i"))
                .wrap(
                    ErrorHandlers::new()
                        .handler(http::StatusCode::INTERNAL_SERVER_ERROR, routes::handle_500),
                )
                .wrap(
                    middleware::DefaultHeaders::new()
                        .header("Access-Control-Allow-Origin", "*")
                        .header("Access-Control-Request-Method", "GET, POST, PUT, DELETE, OPTIONS")
                        .header(
                            "Access-Control-Allow-Headers",
                            "Authorization, Origin, Accept, Accept-Language, Content-Language, Content-Type, *"
                        )
                        .header("Server", "tr_logic_llc_test")
                )
                //.route("/", routes::index::provide())
                .route("/", cached_route!("./data/public/index.html", "text/html; charset=utf-8"))
                .route("/assets/app.js", cached_route!("./data/public/app.js", "text/javascript; charset=utf-8"))
                .route("/assets/environment.js", cached_route!("./data/public/environment.js", "text/javascript; charset=utf-8"))
                .route("/assets/app.css", cached_route!("./data/public/app.css", "text/css; charset=utf-8"))
                .route(
                    "/images/{filename:.*}",
                    actix_web::web::get()
                        .to_async(resources::images::get_one::route)
                )
                .service(resources::images::provide())
                .route(
                    "/thumbnails/{filename:.*}",
                    actix_web::web::get()
                        .to_async(resources::thumbnails::get_one::route)
                )
                .default_service(resources::default::provide())
        })
        .workers(num_cpus::get() * 4)
        .keep_alive(5)
        .bind(format!("0.0.0.0:{}", {
            settings.get("PORT").map_or("8000".to_string(), |v| v.into())
        }))
        .unwrap()
        .start();

        let _ = tx.send(addr);
        let _ = sys.run();

    });
    let addr = rx.recv().unwrap();

    signal_guard.at_exit(move |sig| {
        println!("\nSignal {} received.", sig);
        let _ = addr
            .stop(true)
            .wait()
            .map(|_| println!("Completed successfully!"));
    });
}

#[macro_export]
macro_rules! cached_route {
    ($value:expr, $content_type:expr) => (
        {
            use actix_web::{web, HttpResponse};
            use std::io::Read;

            let data = bytes::Bytes::from({
                let mut data: Vec<u8> = Vec::new();
                std::fs::File::open($value).unwrap().read_to_end(&mut data).unwrap();
                data
            });

            web::get().to(move || {
                HttpResponse::Ok()
                    .content_type($content_type)
                    .body(data.clone())
            })
        }
    );
}
