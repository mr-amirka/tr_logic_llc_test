#[cfg(test)]

extern crate test;

#[test]
pub fn it_route_index() {
    use futures::{Future};
    use actix_web;
    use routes;

    let address = "0.0.0.0:8300";

    let (tx, rx) = std::sync::mpsc::channel();
    let _ = std::thread::spawn(move || {
        let sys = actix_rt::System::new("route_index-server");

        let addr = actix_web::HttpServer::new(|| {
            actix_web::App::new()
                .route("/", routes::index::provide())
        })
            .bind(address)
            .unwrap()
            .start();

        let _ = tx.send(addr);
        let _ = sys.run();
    });
    let addr = rx.recv().unwrap();

    let _ = actix_rt::System::new("route_index").block_on(futures::lazy(|| {
        actix_web::client::Client::default()
            .get(format!("http://{}/", address))
            .send()
            .then(move |r| {
                let mut response = r.unwrap();

                assert_eq!(response.status(), actix_web::http::StatusCode::OK);
                assert_eq!(
                    response.headers().get("content-type").unwrap().as_bytes().starts_with(b"text/plain"),
                    true
                );

                return Box::new(
                    response.body().then(move |r| {
                        assert_eq!(&r.unwrap()[..], b"Home");

                        let _ = addr
                            .stop(true)
                            .wait();

                        Ok::<(),()>(())
                    })
                ) as Box<dyn Future<Item = (), Error = ()>>;
            })
    }));
}

#[test]
pub fn it_route_404() {
    use futures::{Future};
    use actix_web;
    use routes;

    let address = "0.0.0.0:8301";

    let (tx, rx) = std::sync::mpsc::channel();
    let _ = std::thread::spawn(move || {
        let sys = actix_rt::System::new("route_404-server");

        let addr = actix_web::HttpServer::new(|| {
            actix_web::App::new()
                .route("/{path:.*}", actix_web::web::get().to(routes::route_404))
        })
            .bind(address)
            .unwrap()
            .start();

        let _ = tx.send(addr);
        let _ = sys.run();
    });
    let addr = rx.recv().unwrap();

    let _ = actix_rt::System::new("route_404").block_on(futures::lazy(|| {
        actix_web::client::Client::default()
            .get(format!("http://{}/any", address))
            .send()
            .then(move |r| {
                let mut response = r.unwrap();

                assert_eq!(response.status(), actix_web::http::StatusCode::NOT_FOUND);
                assert_eq!(
                    response.headers().get("content-type").unwrap().as_bytes().starts_with(b"text/plain"),
                    true
                );

                return Box::new(
                    response.body().then(move |r| {
                        assert_eq!(&r.unwrap()[..], b"Not found");

                        let _ = addr
                            .stop(true)
                            .wait();

                        Ok::<(),()>(())
                    })
                ) as Box<dyn Future<Item = (), Error = ()>>;
            })
    }));
}


#[test]
pub fn it_set_default_routes_method_options() {
    use futures::{Future};
    use actix_web;
    use routes;

    let address = "0.0.0.0:8302";

    let (tx, rx) = std::sync::mpsc::channel();
    let _ = std::thread::spawn(move || {
        let sys = actix_rt::System::new("set_default_routes_options-server");

        let addr = actix_web::HttpServer::new(|| {
            actix_web::App::new().service(
                routes::set_default_routes(actix_web::web::resource("/path"))
            )
        })
            .bind(address)
            .unwrap()
            .start();

        let _ = tx.send(addr);
        let _ = sys.run();
    });
    let addr = rx.recv().unwrap();

    let _ = actix_rt::System::new("set_default_routes_options").block_on(futures::lazy(|| {
        actix_web::client::Client::default()
            .options(format!("http://{}/path", address))
            .send()
            .then(move |r| {
                let mut response = r.unwrap();

                assert_eq!(response.status(), actix_web::http::StatusCode::OK);
                assert_eq!(response.headers().get("content-type"), None);

                return Box::new(
                    response.body().then(move |r| {
                        assert_eq!(r.unwrap().len(), 0);

                        let _ = addr
                            .stop(true)
                            .wait();

                        Ok::<(),()>(())
                    })
                ) as Box<dyn Future<Item = (), Error = ()>>;
            })
    }));
}


#[test]
pub fn it_set_default_routes_method_not_allowed() {
    use futures::{Future};
    use actix_web;
    use routes;

    let address = "0.0.0.0:8303";

    let (tx, rx) = std::sync::mpsc::channel();
    let _ = std::thread::spawn(move || {
        let sys = actix_rt::System::new("set_default_routes_MethodNotAllowed-server");

        let addr = actix_web::HttpServer::new(|| {
            actix_web::App::new().service(
                routes::set_default_routes(actix_web::web::resource("/path"))
            )
        })
            .bind(address)
            .unwrap()
            .start();

        let _ = tx.send(addr);
        let _ = sys.run();
    });
    let addr = rx.recv().unwrap();

    let _ = actix_rt::System::new("set_default_routes_MethodNotAllowed").block_on(futures::lazy(|| {
        actix_web::client::Client::default()
            .post(format!("http://{}/path", address))
            .send()
            .then(move |r| {
                let mut response = r.unwrap();

                assert_eq!(response.status(), actix_web::http::StatusCode::METHOD_NOT_ALLOWED);
                assert_eq!(
                    response.headers().get("content-type").unwrap().as_bytes().starts_with(b"text/plain"),
                    true
                );

                return Box::new(
                    response.body().then(move |r| {
                        assert_eq!(&r.unwrap()[..], b"Method Not Allowed");

                        let _ = addr
                            .stop(true)
                            .wait();

                        Ok::<(),()>(())
                    })
                ) as Box<dyn Future<Item = (), Error = ()>>;
            })
    }));
}
