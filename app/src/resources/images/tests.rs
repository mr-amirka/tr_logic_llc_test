#[cfg(test)]

extern crate test;


#[test]
fn it_post_one() {
    use std::io::Read;
    use futures::Future;
    use std::str::FromStr;
    use serde_json::{self, Value, Map};
    use actix_rt;
    use actix_web::{self, http, web, guard, HttpServer, App};
    use resources;
    use utils::{app};

    let address = "0.0.0.0:8100";

    let (tx, rx) = std::sync::mpsc::channel();
    let _ = std::thread::spawn(move || {
        let sys = actix_rt::System::new("images_post_one-server");

        let addr = HttpServer::new(|| {
            App::new().service(
                web::resource("/images")
                    .route(
                        web::route()
                            .guard(guard::fn_guard(|req| {
                                req.method == http::Method::POST && {
                                    if let Some(content_type) = req.headers.get("content-type") {
                                        content_type.as_bytes().starts_with(b"image/")
                                    } else {
                                        false
                                    }
                                }
                            }))
                            .to_async(resources::images::post_one::route)
                    )
            )
        })
        .bind(address)
        .unwrap()
        .start();

        let _ = tx.send(addr);
        let _ = sys.run();
    });
    let addr = rx.recv().unwrap();

    let data = {
        let test_file_path = std::path::PathBuf::from_str("./assets/diagram.png").unwrap();
        let mut data: Vec<u8> = Vec::new();

        std::fs::File::open(&test_file_path).unwrap()
            .read_to_end(&mut data).unwrap();

        data
    };

    let _ = actix_rt::System::new("images_post_one").block_on(futures::lazy(|| {

        actix_web::client::Client::default()
            .post(format!("http://{}/images", address))
            .content_type("image/png")
            .send_body(data)
            .then(move |r| {

                let mut response = r.unwrap();

                assert_eq!(response.status(), actix_web::http::StatusCode::OK);
                assert_eq!(
                    response.headers().get("content-type").unwrap().to_str().unwrap(),
                    "application/json"
                );

                return Box::new(
                    response.json::<Value>().then(move |r| {
                        let mut obj_response: Map<String,Value> = serde_json::from_value(r.unwrap()).unwrap();
                        let mut items: Vec<Value> = serde_json::from_value(obj_response.remove("items").unwrap()).unwrap();
                        let mut obj_result: Map<String,Value> = serde_json::from_value(items.pop().unwrap()).unwrap();
                        let mut obj: Map<String,Value> = serde_json::from_value(obj_result.remove("Ok").unwrap()).unwrap();
                        let name: String = serde_json::from_value(obj.remove("name").unwrap()).unwrap();

                        assert_eq!(&name[..], "n/a/s/8a1lMVRM7K4jKjOEmyxsfE0negp5ASFyKL8xjrNQ.png");

                        std::fs::remove_file(app::get_images_dir().join(&name)).unwrap();
                        std::fs::remove_file(app::get_thumbnails_dir().join(&name)).unwrap();

                        assert_eq!(obj.remove("size"), Some(Value::Number(1262.into())));

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
fn it_post_json_string() {
    use futures::Future;
    use serde_json::{self, Value, Map};
    use actix_rt;
    use actix_web:: {self, http, web, guard, HttpServer, App};
    use resources;
    use utils::{app};

    let address = "0.0.0.0:8101";

    let (tx, rx) = std::sync::mpsc::channel();
    let _ = std::thread::spawn(move || {
        let sys = actix_rt::System::new("images_post_json_string-server");

        let addr = HttpServer::new(|| {
            App::new().service(
                web::resource("/images")
                    .route(
                        web::route()
                            .guard(guard::fn_guard(|req| {
                                req.method == http::Method::POST && {
                                    if let Some(content_type) = req.headers.get("content-type") {
                                        content_type.as_bytes().starts_with(b"application/json")
                                    } else {
                                        false
                                    }
                                }
                            }))
                            .to_async(resources::images::post_json::route)
                    )
            )
        })
        .bind(address)
        .unwrap()
        .start();

        let _ = tx.send(addr);
        let _ = sys.run();
    });
    let addr = rx.recv().unwrap();

    let base64_image = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABgAAAAYCAYAAADgdz34AAAFaElEQVR42rVWa2wUVRT+Znd2t93tsttuSx+6LXYphYIIQrWIFWKENqYNQoOAQUlR4y8TUGpjkUaK1gAGfqgRCJEEiUYTSQlgJWiIEKpiE95tgL7ptrS03ffu7M7cGc/Mlh+TIGKMk0xm9t5zznfOd79zZjn8zxf3MEbKmop8SEodUsxvQZIASd4PntvNfXdq4D8BKGsrHWDKepj4Bsxy56F0bnLjzytA5+0hJSE2J2TlSMrR04F/DUBZr4DBUA+HbREsKcCrK4EFq5Kb7T8AR1rAhBiEsdBvChN32lvOHHsoAHl1RSGnYAdSLS8hy2GF1ZLcsJoBd07yfeAOEEtorywiIDY8Fo2Foi1MVrbltp7ruS+AULPcZQRqOaul0VD8qJ3LTqfFODAaJM4ZkEZVZNipNAXwhYGwAPDk4aI1ixmCdwzBaz0hISo0SUw+5Pnl93EdQGxt5XHeYavi3VOBTU2ALROIR4DPt1LGZFtTATy3Pml89mvg6GmqKINSaiBg8rnTC7Z3OwLdQ/CHIyc8p9qqdQDhyvKIzZ1pRXE+MM0NZGUDd0eA67eAKNHhdgFF05MV3OoCBicAE1VQ/BhVkQV4vcDIXaBvCAMd/dGCtos2HUBoSak/zRd24JnZ5FQIBIka4hdGA2AgM4oLCz9ZLgHKMlVIko2TTSrRl/8IpI5bYBduYMhkCBS2X3PqACaWlPrSA1GnxndRHrC0lDKk4N5RQqcglYuB5zcmjVsPAid/Tp7BjAIwRwaE1rNQrvZAMfEYNBn8JRc70nUAo4vn+7IiCafC0VJUgJyeBsOKJeBmFgPdXcks55UmjdvOE0XDQNlCchyFdLUTobOXIZlIabKCYbPR/8TlTj2At2yeb2os4ZRU6akLCQkKk8AtnYeUNzbCGCTltJ6kauj5FAWeXYJQy3HE2juJ92EIPA9OTY6oGyGA0ms39QDdCx/3TRVEp6iJW9YM1WqYSCuePEz5oA6WJ5cmm5DOZ+TDzbDMXQRzbg5667dpUraaTeouhjjOX36jRw/QMb/ElylITkYqMRAdBAExGCKeeShiAnx+Dlz7v4QxqxADZQtgqq6Ee8tW+M60wtuwHanpTsRDIU0Mw0b4X7jZpwe4NHemL0OgMyAL+7rVSC0vh/+LfYhcvALZzEMKR5GztxGmzBz0vb4JVgJIK5mFgX0H4XmzVquk6/1GSHERw7zBX909oAe4MLvI54oTRaIEx4vLkPPxLgiX2tFf9x7EkQnNJvPtddSAVtxuPgDRYIQcj6Ngwxq4323QKrlet02jdtTM+2v6vXqAc8WF/gxJdshMO2JkrKqCY9ly9H70CYTefuoFHunVz4IjgNFvf4LMGSgWQ15NtVZJz4FDeKSqAmnTPfh172eBlZc69H3QOmNaJDfBrBKSPUWjGCKBGUh6spGDTIc4pWwOeXAYb7usASqKDJnsJBJC0cZX4Nlcj4nTP+JE867ohq4+fSd/M2PacXuCVdEpw6BOVTpstRb1sNVA6m2b46Hm4xFQpUkdriYiqYqja9aGtTAV5OP8/q/g7R88URcI6WfRYY/bJTGl1qQojVMkZrcpaj8oSVnSQyRuU7Nd2vyJDo5O7iQTUJsrTMATTAxFYvEmo9l8qH5sQj9N71373LklMpMbiZgVToYUy2QQmYLw1N2afH2hSVcFEVofV2Qhxtgxoq9pqz/Y8cAPzr1rT7brZYr2Dk37p1UyjeptS9VoY6Eo1Ib0U1VBmf0hKcqe7cHw9/eL88Bv8k5Xuosx+TUKvsXOcXl2tVPpxwSN7yCThuIyPiUxHd4Rioz/XYyH+lfR5LBPFyW5jkZZrUQKEOmLRZ67myOxrn/y/QueLmc39WrjeAAAAABJRU5ErkJggg==";

    let req_json_data = Value::String(base64_image.into());

    let _ = actix_rt::System::new("images_post_json_string").block_on(futures::lazy(|| {

        actix_web::client::Client::default()
            .post(format!("http://{}/images", address))
            .content_type("application/json")
            .send_body(serde_json::to_string(&req_json_data).unwrap())
            .then(move |r| {

                let mut response = r.unwrap();

                assert_eq!(response.status(), actix_web::http::StatusCode::OK);
                assert_eq!(
                    response.headers().get("content-type").unwrap().to_str().unwrap(),
                    "application/json"
                );

                return Box::new(
                    response.json::<Value>().then(move |r| {
                        let mut obj_response: Map<String,Value> = serde_json::from_value(r.unwrap()).unwrap();
                        let mut items: Vec<Value> = serde_json::from_value(obj_response.remove("items").unwrap()).unwrap();
                        let mut obj_result: Map<String,Value> = serde_json::from_value(items.pop().unwrap()).unwrap();
                        let mut obj: Map<String,Value> = serde_json::from_value(obj_result.remove("Ok").unwrap()).unwrap();
                        let name: String = serde_json::from_value(obj.remove("name").unwrap()).unwrap();

                        assert_eq!(&name[..], "C/7/f/_ExK8vUSqp9Rxw_u79bPKisi4CrjI_dvtCyxM2sw.png");

                        std::fs::remove_file(app::get_images_dir().join(&name)).unwrap();
                        std::fs::remove_file(app::get_thumbnails_dir().join(&name)).unwrap();

                        assert_eq!(obj.remove("size"), Some(Value::Number(1441.into())));

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
fn it_post_json_object() {
    use futures::Future;;
    use serde_json::{self, Value, Map, json};
    use actix_rt;
    use actix_web:: {self, http, web, guard, HttpServer, App};
    use resources;
    use utils::{app};

    let address = "0.0.0.0:8102";

    let (tx, rx) = std::sync::mpsc::channel();
    let _ = std::thread::spawn(move || {
        let sys = actix_rt::System::new("images_post_json_object-server");

        let addr = HttpServer::new(|| {
            App::new().service(
                web::resource("/images")
                    .route(
                        web::route()
                            .guard(guard::fn_guard(|req| {
                                req.method == http::Method::POST && {
                                    if let Some(content_type) = req.headers.get("content-type") {
                                        content_type.as_bytes().starts_with(b"application/json")
                                    } else {
                                        false
                                    }
                                }
                            }))
                            .to_async(resources::images::post_json::route)
                    )
            )
        })
        .bind(address)
        .unwrap()
        .start();

        let _ = tx.send(addr);
        let _ = sys.run();
    });
    let addr = rx.recv().unwrap();

    let image_base64_content = "iVBORw0KGgoAAAANSUhEUgAAABgAAAAYCAYAAADgdz34AAAE/klEQVR42p1WDUxTVxT+XluoSB+l4BBFdENZmIZfoRAbxIFRtxll08Rsy+aE4aZGZpwy1A6nsGXR+I9CdMl+sowtM5G5LCO6YAIWBVFEEFp1Ai1CKS2lrQjS17LznpTNID/zJM27uff2+757zrnnXAbj2KHjhezAwECi08mlGLu6ImnqheGl7ulBQQ0SiaRC6i29mrM92zEWBvOsya8PHZVYLD2bem22LLfbPZ+AxDNnBMOPZTE0NIReux1dXSZwHOcSiURNNP+Nv7/81Oe5O7kJCXbv3R9tNJnODLmHEl4On4tlaalY8EoEiAROpxNOjiNgF/r6HkGru4PKK1egNxggYphr0wIDsw58ub9+TIKdu9WJDzo6S2W+vsGZH7yPmOgocE4OdsdD9A8MCJtFIjHEYhF9RQKh42EfmrVa/HnhAsidRn+5PP3UscPVowg+zd0TbWhvL5szOzT4ky2bMS0wAJ1GEx6SUrFYDC86Af8lbwnfJz+ehIPZ0gMDneKPsjIaW4xTfXxWfHemuH6EQP1FvuTuvb810ilS5Vf78qDwV6BN3w6X2/0EVACWjAIX86ehOYoTKis1GHg8iHPnz2PQOVhDcVF9e7qIEwgyPtq8tcdqPb5jWzbi42Jxv0U/Aj6Wcs94iPYdOXoUJSU/Y2fOZ3TiPlyqqIC3l1f22ZIfTzA7du1hm7V3NAvmR0Tm56nR0dk14paJlPPrBw4eRHFxMZ9RiImJRUbmh7hQXg5Td3djoEKxiMnasnVpS2tb2T71bnF0VCRaWg3/S3lhYeFTWZi9bTs5XoTKKo1LzvqtYN7dkJlv6jarz/3yE2w2B2x2x6SVF5FyF/d06icvTkHa0mU49/t5ULALGFXqstJ5c8NWFx8/gg7KGrfL/VzKPTYzJITctBEXL5WTYPtvTFJKqiZh4cJFBXvzYOmxCpsmVF5Eyl3cMwkCAgKxPiMTNbW1fByqmHhVskCQn5cnBFckYp5LuccUigC889561N4YJohLUpWGhb20+sThQ8KlGV95ESl3jUsQND0Ya9e9jcrLlbDbyUXLV67K7zZb1CU/fA8fHx94e3s/l3KPRcXGYWG8Ehf/ugifKVMKmNdXpS9t1evLduXkiBMTlHRE+dPKD/DZMrFyj61Mfwsc7a2ru+FiZewK5s2161itTquJiIiIzKWbKJRlP/aJ8iOk/OTklPMWOudFJC9JxbWaq7BarY0Uj0VCqUhSJQulYsumzUhKTEJ4eBiOkVuEPJ+kcl9fGdKWv0Z+t6H+Zh2kUmn27fq6EwLBG6vSJTqdTkP+V+bm5NKiN9TqPejpsUwaPPnVVIqZBFWXK3hRNf4KharuWjU3Uq5Vi5dEt7W1loXMDAnesCFDqPVnz/6KxoZb44LPmj0HMXHxVBxduF5TDYfDbpSR77W3G/4t1x6Li1cmdnZ2lE6d6hu8Zs1azJ0XDn1bG+rrb6Kl5T4eU9PhW6YXZdr0GTMQSuB+cn+0G/S4eb2WRA0aZaxf+l1t0+iGM5JmMbHRZrP5NAEpZxNAgjIRs0JDCRh49KiPaj0HGqK/vx8P2g3QNTfBYu7m/1ojY9mN93TasVumx+KVSRLy/8cOh2MjEc2n+yDmSwB/T9zE1Ec139Zr5RuNi2GYJpo/7Stji5sbb03c9P9r5Ft6tvTTs8W5uNdq5Z8tQcNLJj+5vEFMzxZKjOqmhltjPlv+Acf+V80Wn9LCAAAAAElFTkSuQmCC";

    let req_json_data = serde_json::to_string(&json!({
        "type": "image/png",
        "content": image_base64_content
    })).unwrap();

    let _ = actix_rt::System::new("images_post_json_object").block_on(futures::lazy(|| {

        actix_web::client::Client::default()
            .post(format!("http://{}/images", address))
            .content_type("application/json")
            .send_body(req_json_data)
            .then(move |r| {

                let mut response = r.unwrap();

                assert_eq!(response.status(), actix_web::http::StatusCode::OK);
                assert_eq!(
                    response.headers().get("content-type").unwrap().to_str().unwrap(),
                    "application/json"
                );

                return Box::new(
                    response.json::<Value>().then(move |r| {
                        let mut obj_response: Map<String,Value> = serde_json::from_value(r.unwrap()).unwrap();
                        let mut items: Vec<Value> = serde_json::from_value(obj_response.remove("items").unwrap()).unwrap();
                        let mut obj_result: Map<String,Value> = serde_json::from_value(items.pop().unwrap()).unwrap();
                        let mut obj: Map<String,Value> = serde_json::from_value(obj_result.remove("Ok").unwrap()).unwrap();
                        let name: String = serde_json::from_value(obj.remove("name").unwrap()).unwrap();

                        assert_eq!(&name[..], "4/_/E/qX01hi5f5rvxjEwbwZ0jWJ26aVXIFiXTexpH4tp0.png");

                        std::fs::remove_file(app::get_images_dir().join(&name)).unwrap();
                        std::fs::remove_file(app::get_thumbnails_dir().join(&name)).unwrap();

                        assert_eq!(obj.remove("size"), Some(Value::Number(1335.into())));

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
fn it_post_json_array() {
    use futures::Future;
    use serde_json::{self, Value, Map, json};
    use actix_rt;
    use actix_web:: {self, http, web, guard, HttpServer, App};
    use resources;
    use utils::{app};

    let address = "0.0.0.0:8103";

    let (tx, rx) = std::sync::mpsc::channel();
    let _ = std::thread::spawn(move || {
        let sys = actix_rt::System::new("images_post_json_array-server");

        let addr = HttpServer::new(|| {
            App::new().service(
                web::resource("/images")
                    .route(
                        web::route()
                            .guard(guard::fn_guard(|req| {
                                req.method == http::Method::POST && {
                                    if let Some(content_type) = req.headers.get("content-type") {
                                        content_type.as_bytes().starts_with(b"application/json")
                                    } else {
                                        false
                                    }
                                }
                            }))
                            .to_async(resources::images::post_json::route)
                    )
            )
        })
        .bind(address)
        .unwrap()
        .start();

        let _ = tx.send(addr);
        let _ = sys.run();
    });
    let addr = rx.recv().unwrap();


    let image1_base64_content = "iVBORw0KGgoAAAANSUhEUgAAABgAAAAYCAYAAADgdz34AAADiklEQVR42u2UWUwTURSG/wm2VQShWJaCRMU1bk/6YOIeHzQqQqOJ+uASHwwajWsEd9wNRPTBLS5RX/QJYhQ1RsUtUaNG4xISsQqKBYUOtKUtlGHGc+8M7UwoEh588ySn9/TOnf8799wzV8A/NuE/oFcAx+5yJeoKpWeh0gPzoibLJ98djXebE1KTdrlO4OzW6b3Ock3RY5QdnN894H2RTRm/+Twce2NwestMnrLAHulfUfT7VTpXcMsrqkDZoeweAAsz4LiyH6e2zOAv5x2502PmZwrmcMy64goUWldj4MS1czNm7r/bDSAduZcLcXLzLHVSoZQFXVL6//rdULyh5CHKdn7Fh9NHGGQ4QZxRATmX9qGEAH87Z0FjhWP6OXjyOoodz1H7sJxNd5DHaMtHRgA5diy8sAfFG2fzJ1uP3zNAYui9QaZqDDY5kWn6hhHmSvQR2sPP+9lSYUm0wpKQACnghbuykk2nRQC5acg+txvH8iZBrimH4n4PiJ+geD4bQP2SM2FJSoHFaiOxRO59EwZAkUOA3AZBCcF56wYCjW5M2NYo6ADpWHC2AIdT16tCKUNIKB2WgXYSSSZBEk2M4/URSEiRA0CrC0LLW5qTwwn4fGmovv+AheMJ8DECWDwWq85kIz/rAkat2EW1NUPoECkrakmhL41B+u8B2t3kojpKfhIPGXbofFrPsneReIbxkJdNwbWLPkyfnAL71By+VTBBuVUV6mgh91HMRsqe7aCjFfrP3OdNRvWDRyy0EcBtBCxfgA9XbyJr8Q7E2jMhSE2qCBOXNQAX7wQEySVj9k/qEHCLVSQ+skubjlkyB1XlrzB65V71O5VEVUTSiXOAX51X2iL9SuZttqKm4hkL4wjg7wKwjcmCYh5G5XFQrZvVcvAy0FrJp0H8ava8NMbsvzxxIehuekPiE6N+yWzMWrQdsWmDKfv6SJ25uAbhwCBvR715m+JR8+gFC80EaI8KMMUnUfcU0uGyQxU10c6aa3H4YGVj9o9rERQ9r0l8UrRbAB9L7Ip13DTYp2Sr5ZG8qigvi7YD7m28LT0NZnh+NMD/uwFSsLVTy0KAUFQA7UAemrtJ6J9B5QnVq13DIQG0iH0QqKtG4PdP+H/VQW4PV6CWnF0+7E65z7qUAEp3AGX40nyYYuNIrAp+lxOB+u/cNXtJzu7vuyTyEr0wwyFrVkZeSn6bxMTeiEWzPySr10bP2MDrAAAAAElFTkSuQmCC";

    let image2_base64_content = "iVBORw0KGgoAAAANSUhEUgAAABgAAAAYCAYAAADgdz34AAACiUlEQVR42q2V604TQRSAe3bbjC9QvEAbURSfwEh/K5qYaCDtQuI9hGA01RSIQvlNQUPbaKPRELyTANtoNDHx9r88AtUqhhYv9AFMbLvLmWG2maWkO1VO0pyvzcx8M3NmpuCSjLFPh/5g2jF+dAk4E2TFqR/ICqIfNwSxY0vAmSBvn2D0fTsTTBzPAmeC/H8C3IoWTB34CVVKRjdmVfUousCDmDO4VYsNC6If2iOYEtb3cslk2e0BG/PQY51ZTVow8s4+OA2cNcs4axsLkZg8kR2SEtx4e3AFk0/sXCkbYcye+OkcDL1u+0sXoLoVsX/+9snPfinB8JsDpvBVnzr1RYu83M+KnOz+CpwJCtK0PlZDbAdSgsFXbaJAS3Tl9OvpfUxwJ/gNOBNVVeiWdFSX2ZXTpAQ4gG0FOKgWnm9lglTPMnAmyP92TMNzrZtrkDEM8zBmVVFAQ56lNUAOpHqXFxsWXHmxt+YUmebGogDAxhh5/CTvn/2elBbQGHjij9NyOG2BEImHF1fkjqkV/TO+6k3Gluz2ukycMfDtM2u6+Kb78gVpgRh90y2syDP9BeBMTMNlO6YY2qOBgl5XcOFeszVrKzJPr64WLj1oZoLHl1eBMzEqLj+vQVWAbesLzt3dfYQOKnZ6fu2njmImwAGAMwqMwFZtHbfoTHKX7R7MRn5p51N7mOBZ+AdwJvh81GwRtnUW9E7t3HwPXG63QqVQLhs6cpBzTZHnhn87F7nnVhOd1YL4m/Vy0pdUZCES8zfX5I9paMJLJXFrJXUE7KLpo8XGLpoVwXEvLbpP9QB9HjyVkqlxdiMH0mPFxp+KrUKbbGJFXhhZA84Eefv+9EMxLxPo0SJwJsiOgnWQ2xgoJxfaEAAAAABJRU5ErkJggg==";

    let base64_image = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABgAAAAYCAYAAADgdz34AAACeUlEQVR42mNkIBJU7lIXBVJyQKwCFboDxI/a3W6+xqePkYChbEAqDYhDgNgeh7KDQLwGiGcBLftFtAUVO9W9gdQEJBcTAveBuKDD/eYmghaUb1erAFLtxAYfusc7PW914LSgbKtqKZDqQhN+B8TLgHg3ED+FiikCsTc0+HjQjenyvt2NYUHJZhVXILUdiJmRhPuAuKnH985HbM4F6hGFBmUUkvBfIPYE6tkNt6BogwrI0KtArI6kMKUv4M5cYsIFqL8RSNUhCd0EYm24BYXrlBOA1HwkBRP7g+4WkBL4QDPWAalAJKFExqrd6v/+/2dg+P/vPyMQM/z7B2YD6f////0F0wxgGoIZkcRA8h8XZD4TxGcpyIL/UAsY0CxgwGIButhWoAU+hCz4DrSA6f/f/2wgm5As+A004C+SYcxAzIpkwQ8grgJa0I/XAhCRuVgelNy2IIkrTY99eJ+U8MdrQfp8OQ8GSPKEAdWZiY/uUMUCCiL5AxCLAIPoL60ieeu8tKc+xPgAFMkgDWwM//8zkRDJpUALeoiKAxCImyIJKlNWI8l1LMp5XklKeAPNWAGkwpGEQlHKopiJEmeAlDGUCwrb9CX5L4gqKoB60YuKs0C9JigWRPeLgww/BsRsSMLgwm5p4UushR1QD7bCDlTxWAH1nMUoriN7xGOB1CI0YVhxvRWIYVWkNAOk3MFWXMctL3m5GCUOkEF4lxjINXOAmJOUOACC76BgXVn2ajFMAGeVGdYhagCkpgGxJZGGHwfirFUVry8gCzIS0hXaJgLK5clADKqM+NGkQfECquXmrq56swObfoIWIIOQVhFQs0UUyn29pvrNI0J6AMZQqHLAJvkrAAAAAElFTkSuQmCC";

    let req_json_data = Value::Array(vec![
        json!({ // object
            "type": "image/png",
            "content": image1_base64_content
        }),
        json!({ // object
            "type": "image/png",
            "content": image2_base64_content
        }),
        Value::String(base64_image.into()), // string
        json!({}) // empty object - invalid
    ]);

    let _ = actix_rt::System::new("images_post_json_array").block_on(futures::lazy(|| {

        actix_web::client::Client::default()
            .post(format!("http://{}/images", address))
            .content_type("application/json")
            .send_body(serde_json::to_string(&req_json_data).unwrap())
            .then(move |r| {

                let mut response = r.unwrap();

                assert_eq!(response.status(), actix_web::http::StatusCode::OK);
                assert_eq!(
                    response.headers().get("content-type").unwrap().to_str().unwrap(),
                    "application/json"
                );

                return Box::new(
                    response.json::<Value>().then(move |r| {
                        let mut obj_response: Map<String,Value> = serde_json::from_value(r.unwrap()).unwrap();
                        let mut items: Vec<Value> = serde_json::from_value(obj_response.remove("items").unwrap()).unwrap();

                        let four = items.pop().unwrap();
                        let three = items.pop().unwrap();
                        let two = items.pop().unwrap();
                        let one = items.pop().unwrap();

                        {
                            let mut result: Map<String,Value> = serde_json::from_value(one).unwrap();
                            let mut obj: Map<String,Value> = serde_json::from_value(result.remove("Ok").unwrap()).unwrap();
                            let name: String = serde_json::from_value(obj.remove("name").unwrap()).unwrap();

                            assert_eq!(result.remove("Err"), None);
                            assert_eq!(&name[..], "U/O/z/lpkq_fQkGVIcSk3YpQCUtMu1U7E9Df0QcFFAZ514.png");

                            std::fs::remove_file(app::get_images_dir().join(&name)).unwrap();
                            std::fs::remove_file(app::get_thumbnails_dir().join(&name)).unwrap();

                            assert_eq!(obj.remove("size"), Some(Value::Number(963.into())));
                        }
                        {
                            let mut result: Map<String,Value> = serde_json::from_value(two).unwrap();
                            let mut obj: Map<String,Value> = serde_json::from_value(result.remove("Ok").unwrap()).unwrap();
                            let name: String = serde_json::from_value(obj.remove("name").unwrap()).unwrap();

                            assert_eq!(result.remove("Err"), None);
                            assert_eq!(&name[..], "3/K/7/t5WzwZ-xZBU5c-0xRNDsbTwNFLReGLx6tqGf2XlY.png");

                            std::fs::remove_file(app::get_images_dir().join(&name)).unwrap();
                            std::fs::remove_file(app::get_thumbnails_dir().join(&name)).unwrap();

                            assert_eq!(obj.remove("size"), Some(Value::Number(706.into())));
                        }
                        {
                            let mut result: Map<String,Value> = serde_json::from_value(three).unwrap();
                            let mut obj: Map<String,Value> = serde_json::from_value(result.remove("Ok").unwrap()).unwrap();
                            let name: String = serde_json::from_value(obj.remove("name").unwrap()).unwrap();

                            assert_eq!(result.remove("Err"), None);
                            assert_eq!(&name[..], "T/W/a/Q2fz3QhsVX-j_ikVzQ3gE9XXatKJNpzQK_wdOKX8.png");

                            std::fs::remove_file(app::get_images_dir().join(&name)).unwrap();
                            std::fs::remove_file(app::get_thumbnails_dir().join(&name)).unwrap();

                            assert_eq!(obj.remove("size"), Some(Value::Number(690.into())));
                        }
                        {
                            let mut result: Map<String,Value> = serde_json::from_value(four).unwrap();

                            assert_eq!(result.remove("Ok"), None);
                            assert_eq!(result.remove("Err"), Some(json!({
                                "type": "Invalid"
                            })));
                        }

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
fn it_post_multipart() {
    use std::io::Read;
    use futures::Future;
    use std::str::FromStr;
    use serde_json::{self, Value, Map, json};
    use actix_rt;
    use actix_web:: {self, http, web, guard, HttpServer, App};
    use resources;
    use utils::{app};

    let address = "0.0.0.0:8104";

    let (tx, rx) = std::sync::mpsc::channel();
    let _ = std::thread::spawn(move || {
        let sys = actix_rt::System::new("images_post_multipart-server");

        let addr = HttpServer::new(|| {
            App::new().service(
                web::resource("/images")
                    .route(
                        web::route()
                           .guard(guard::fn_guard(|req| {
                               req.method == http::Method::POST && {
                                   if let Some(content_type) = req.headers.get("content-type") {
                                       content_type.as_bytes().starts_with(b"multipart/form-data")
                                   } else {
                                       false
                                   }
                               }
                           }))
                           .to_async(resources::images::post_multipart::route)
                    )
            )
        })
        .bind(address)
        .unwrap()
        .start();

        let _ = tx.send(addr);
        let _ = sys.run();
    });
    let addr = rx.recv().unwrap();

    let boundary = "----hjidhakdpjkn773rqjhhnas8sd";
    let boundary_line = format!("--{}\r\n", boundary);
    let boundary_end = format!("--{}--\r\n", boundary);
    let mut data: Vec<u8> = Vec::new();
    // file "./assets/green_0000.png"
    {
        data.extend_from_slice(boundary_line.as_bytes());
        data.extend_from_slice(b"Content-Disposition: form-data; name=\"image\"; filename=\"green_0000.png\"\r\n");
        data.extend_from_slice(b"Content-Type: image/png\r\n\r\n");

        let test_file_path = std::path::PathBuf::from_str("./assets/green_0000.png").unwrap();

        std::fs::File::open(&test_file_path).unwrap()
            .read_to_end(&mut data).unwrap();
        data.extend_from_slice(b"\r\n");
    }
    // file "./assets/green_0001.jpg"
    {
        data.extend_from_slice(boundary_line.as_bytes());
        data.extend_from_slice(b"Content-Disposition: form-data; name=\"image\"; filename=\"green_0001.jpg\"\r\n");
        data.extend_from_slice(b"Content-Type: image/jpeg\r\n\r\n");

        let test_file_path = std::path::PathBuf::from_str("./assets/green_0001.jpg").unwrap();

        std::fs::File::open(&test_file_path).unwrap()
            .read_to_end(&mut data).unwrap();
        data.extend_from_slice(b"\r\n");
    }
    // invalid field
    {
        data.extend_from_slice(boundary_line.as_bytes());
        data.extend_from_slice(b"Content-Disposition: form-data; name=\"text\"\r\n");
        data.extend_from_slice(b"Content-Type: text/plain\r\n\r\n");
        data.extend_from_slice(b"hello!\r\n");
    }
    data.extend_from_slice(boundary_end.as_bytes());

    let _ = actix_rt::System::new("images_post_multipart").block_on(futures::lazy(|| {

        actix_web::client::Client::default()
            .post(format!("http://{}/images", address))
            .content_type(format!("multipart/form-data; boundary={}", boundary))
            .send_body(data)
            .then(move |r| {
                let mut response = r.unwrap();

                assert_eq!(response.status(), actix_web::http::StatusCode::OK);
                assert_eq!(
                    response.headers().get("content-type").unwrap().to_str().unwrap(),
                    "application/json"
                );

                return Box::new(
                    response.json::<Value>().then(move |r| {
                        let mut obj_response: Map<String,Value> = serde_json::from_value(r.unwrap()).unwrap();
                        let mut items: Vec<Value> = serde_json::from_value(obj_response.remove("items").unwrap()).unwrap();

                        let three = items.pop().unwrap();
                        let two = items.pop().unwrap();
                        let one = items.pop().unwrap();

                        {
                            let mut result: Map<String,Value> = serde_json::from_value(one).unwrap();
                            let mut obj: Map<String,Value> = serde_json::from_value(result.remove("Ok").unwrap()).unwrap();
                            let name: String = serde_json::from_value(obj.remove("name").unwrap()).unwrap();

                            assert_eq!(result.remove("Err"), None);
                            assert_eq!(&name[..], "3/-/p/kzgPZuxt7vsyU0rAo55uuy8S3dFGzoBH18FsD5gI.png");

                            std::fs::remove_file(app::get_images_dir().join(&name)).unwrap();
                            std::fs::remove_file(app::get_thumbnails_dir().join(&name)).unwrap();

                            assert_eq!(obj.remove("size"), Some(Value::Number(158425.into())));
                        }
                        {
                            let mut result: Map<String,Value> = serde_json::from_value(two).unwrap();
                            let mut obj: Map<String,Value> = serde_json::from_value(result.remove("Ok").unwrap()).unwrap();
                            let name: String = serde_json::from_value(obj.remove("name").unwrap()).unwrap();

                            assert_eq!(result.remove("Err"), None);
                            assert_eq!(&name[..], "e/J/u/mzhiaCaikEE9DK0-ChhuFGROrulnRkQpqZee2RMk.jpg");

                            std::fs::remove_file(app::get_images_dir().join(&name)).unwrap();
                            std::fs::remove_file(app::get_thumbnails_dir().join(&name)).unwrap();

                            assert_eq!(obj.remove("size"), Some(Value::Number(35695.into())));
                        }
                        {
                            let mut result: Map<String,Value> = serde_json::from_value(three).unwrap();

                            assert_eq!(result.remove("Ok"), None);
                            assert_eq!(result.remove("Err"), Some(json!({
                                "type": "Unsupported"
                            })));
                        }

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
fn it_post_url_text() {
    use futures::Future;
    use std::str::FromStr;
    use serde_json::{self, Value, Map, json};
    use actix_files::NamedFile;
    use actix_rt;
    use actix_web:: {self, http, web, guard, HttpServer, App};
    use resources;
    use utils::{app};

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
        let sys = actix_rt::System::new("images_post_url_text-static-server");

        let addr = actix_web::HttpServer::new(|| {
            actix_web::App::new().route("/assets/{filename:.*}", actix_web::web::get().to(index))
        })
            .bind("0.0.0.0:8105")
            .unwrap()
            .start();

        let _ = tx.send(addr);
        let _ = sys.run();

    });
    let static_server_addr = rx.recv().unwrap();

    let address = "0.0.0.0:8106";

    let (tx, rx) = std::sync::mpsc::channel();
    let _ = std::thread::spawn(move || {
        let sys = actix_rt::System::new("images_post_url_text-server");

        let addr = HttpServer::new(|| {
            App::new().service(
                web::resource("/images")
                    .route(
                        web::route()
                           .guard(guard::fn_guard(|req| {
                               req.method == http::Method::POST && {
                                   if let Some(content_type) = req.headers.get("content-type") {
                                       content_type.as_bytes().starts_with(b"text/plain")
                                   } else {
                                       false
                                   }
                               }
                           }))
                           .to_async(resources::images::post_url_text::route)
                    )
            )
        })
        .bind(address)
        .unwrap()
        .start();

        let _ = tx.send(addr);
        let _ = sys.run();
    });
    let addr = rx.recv().unwrap();

    let image_urls = r##"http://0.0.0.0:8105/assets/green_0002.jpg
data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABgAAAAYCAYAAADgdz34AAAFT0lEQVR42oVWa2wUVRT+7p2Z7XZ2t7JtgYJCi0XAoAH5gRCwaWtS/wAmopj4Ah9/aMIPjUYTDAQjiSZGfxBLSDSWaEiwmKhIakzaRcqzCBQDEh4V+wjt9rG77G53pjsz93pmdvvCFk92N3vPfOd5v3N2Ge4jsXPQpYPVjKNKUZXlpCqVksG2lCHblFcdWznBoLXPr0tmZvLBplMmzkN3HGwNlC1p8BWvBLQKgJdBsgdyBjIJ2FHYxk0YfX8h2d1TL6V2cOGGeOZ/A1DWK1Qf+zxYWVsr1VUQmA/GQ2BKIRjTPIwUFqQ0AZGmQN1wkucx9OfFVsvQ3ql4Ln55xgDkvDowZ2FEm1sLWywD10rBOTmV9GLMA0vv4FrSSbpfHAh7ACxzCYlb7Uh0xWsaD8jfP269K6cEcDPX5y7qUGc/Q/bLKOOCvEM2nonM5Z8/s1ycMa0wITIdiF8/ibtdsZVLXspV4lnH26FDVY6FFr9cLZXH4dhAKpaGoIvgioJQSRCqqngJjyXuPku6GMsBEQDB4iAUhapJX0LPyV8jjqluWPrKcMYLEDuL7aEldQ2Cr4UQHNHbg/jqvW/HW/fWZ69ibsVsz5FrYNsOov8Q5t3/YjgfRXbgDHpPX65/dFtsP4udgV5QWjmihuuo77Mw0DWIr99vwra9m6EXFSKTNNC48we8+ekLmFNe6gWIdg0T5vsZMVxEEbt2HonO/gAbOs2qg+XrIoIYk4qb2Fd/GG/s3UjAEmoPg+MIChrDNzuPYseXL3pNdTGv791EmDC1hVNLCdMdJ8zPHiYU1mAle9DT2lbDhk/zXQVlj+1hvrVwhICRyqIwpHmGY0Szqd9myvL0ruQwvjwmRwI3ESNN+iDpOYe0U+hsPrabxc+ph5levCURL0fJAmIP3aKU8h4S5zgjJ3OaMB7W03JMICWG+1KYFR5AT9vVIyzRrrXYSnHtqWbaCXUV0MOLxqnoRfCo46rcD5GLmT+7KibZJLyEkbFwvKkDT2+2EO0w2ljsrK/FQrg2MSzwR6uDqmfnIRguo6QKx3NmE2yfMhUSbMqkjtxNoaXpBlattzHnIYb+S6NtbLDNf9hhRVvcJXany8FvR2w883wA8x8ugVIwi9aEb6KKeyU3aTTJJnpvDaH5UBTVm1RULlM8fe8F4wjrayncxbXAHtqMbr0YigKRoxaypsSTtX48WB6AP6RD0+iCmY/aQrNAZLCcLMy0iTu3kzjbkqb7kKjZqKFsAffqdVvXdcHYzXqag9X+gBbJmr7xsq2sRHcnteyUDTPpnqdfxRqZaLrEE2tULKasff589+htGxJ9V7I1rOdYSFcL+Ih0fMTnCTa4oGyaYV7jKtxPel+7SBXKKR1UaH5ivaNID7KAp+/6qWi7P8gbRpMFE/xxCWQz+PYtvW8Aa8d1KiNPB/rgnCGbEei7YdSv35XanwvwY7HO1OwvKvfVjI5oUwKkPim7b4CiD/ohVTmJvrRKbhsROco3rPkwkRmv7O+m8ArNLzqko9IFq14A9+EIJRg7GJzWeXhrGvrS3Ji5hKX7R7zfRGZYrFy3Ozmxrsfk5qFwtVYoIwpXYRk07oJ5GY3cEOjcP5WnldsB/RF3iBltWdpHtE5SQ1l62zVVe9LHJ13lVLn+XckKRRFfFPh5jSDq2llGK5y4PppvW96IK5J+8ci5xmEaNhJ9ZiSbkW8/9VF65p/MMbnWWKoLR2xVNTSoqkqVkGOHYXxF5ckmHImRZBaZuF1PuINuz+/1NW2AMblyoES3bbGa3FfR5CwXQpYK251cOeRYuCodfoIz3r5mZ3zGvy3/AoXYfx61FNCXAAAAAElFTkSuQmCC
http://0.0.0.0:8105/assets/green_0003.jpg
meow"##;

    let _ = actix_rt::System::new("images_post_url_text").block_on(futures::lazy(|| {

        actix_web::client::Client::default()
            .post(format!("http://{}/images", address))
            .content_type("text/plain")
            .send_body(image_urls)
            .then(move |r| {
                let mut response = r.unwrap();

                assert_eq!(response.status(), actix_web::http::StatusCode::OK);
                assert_eq!(
                    response.headers().get("content-type").unwrap().to_str().unwrap(),
                    "application/json"
                );

                return Box::new(
                    response.json::<Value>().then(move |r| {
                        let mut obj_response: Map<String,Value> = serde_json::from_value(r.unwrap()).unwrap();
                        let mut items: Vec<Value> = serde_json::from_value(obj_response.remove("items").unwrap()).unwrap();

                        let four = items.pop().unwrap();
                        let three = items.pop().unwrap();
                        let two = items.pop().unwrap();
                        let one = items.pop().unwrap();

                        {
                            let mut result: Map<String,Value> = serde_json::from_value(one).unwrap();
                            let mut obj: Map<String,Value> = serde_json::from_value(result.remove("Ok").unwrap()).unwrap();
                            let name: String = serde_json::from_value(obj.remove("name").unwrap()).unwrap();

                            assert_eq!(result.remove("Err"), None);
                            assert_eq!(
                                obj.remove("original"),
                                Some(Value::String("http://0.0.0.0:8105/assets/green_0002.jpg".to_string()))
                            );
                            assert_eq!(&name[..], "s/w/-/GLWQedNWoUApJ-OjfGqQ5YhQ4pXu0x0lFHCA_Hfs.jpg");

                            std::fs::remove_file(app::get_images_dir().join(&name)).unwrap();
                            std::fs::remove_file(app::get_thumbnails_dir().join(&name)).unwrap();

                            assert_eq!(obj.remove("size"), Some(Value::Number(106432.into())));
                        }
                        {
                            let mut result: Map<String,Value> = serde_json::from_value(two).unwrap();
                            let mut obj: Map<String,Value> = serde_json::from_value(result.remove("Ok").unwrap()).unwrap();
                            let name: String = serde_json::from_value(obj.remove("name").unwrap()).unwrap();

                            assert_eq!(result.remove("Err"), None);
                            assert_eq!(obj.remove("original"), Some(Value::Null));
                            assert_eq!(&name[..], "I/Z/-/BjhYHGCebfhPoBGSEy6STwxPeH1XxmomLJ38-V0w.png");

                            std::fs::remove_file(app::get_images_dir().join(&name)).unwrap();
                            std::fs::remove_file(app::get_thumbnails_dir().join(&name)).unwrap();

                            assert_eq!(obj.remove("size"), Some(Value::Number(1416.into())));
                        }
                        {
                            let mut result: Map<String,Value> = serde_json::from_value(three).unwrap();
                            let mut obj: Map<String,Value> = serde_json::from_value(result.remove("Ok").unwrap()).unwrap();
                            let name: String = serde_json::from_value(obj.remove("name").unwrap()).unwrap();

                            assert_eq!(result.remove("Err"), None);
                            assert_eq!(
                                obj.remove("original"),
                                Some(Value::String("http://0.0.0.0:8105/assets/green_0003.jpg".to_string()))
                            );
                            assert_eq!(&name[..], "Y/7/J/cvRJdD_IH0shEoe7ajxQNsTN5NAOw_pIyUqSwybw.jpg");

                            std::fs::remove_file(app::get_images_dir().join(&name)).unwrap();
                            std::fs::remove_file(app::get_thumbnails_dir().join(&name)).unwrap();

                            assert_eq!(obj.remove("size"), Some(Value::Number(141014.into())));
                        }
                        {
                            let mut result: Map<String,Value> = serde_json::from_value(four).unwrap();

                            assert_eq!(result.remove("Ok"), None);
                            assert_eq!(result.remove("Err"), Some(json!({
                                "type": "NetError"
                            })));
                        }

                        let _ = static_server_addr
                            .stop(true)
                            .wait();

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
pub fn it_get_one() {
    use futures::{Future};
    use actix_web;
    use utils::{app};
    use resources;

    let address = "0.0.0.0:8107";

    let images_dir = app::get_images_dir();
    let _ = std::fs::create_dir_all(&images_dir);
    let target_path = images_dir.join("green_0005.jpg");
    std::fs::copy("./assets/green_0005.jpg", &target_path).unwrap();

    let (tx, rx) = std::sync::mpsc::channel();
    let _ = std::thread::spawn(move || {
        let sys = actix_rt::System::new("images_get_one-server");

        let addr = actix_web::HttpServer::new(|| {
            actix_web::App::new()
                .route(
                    "/images/{filename:.*}",
                    actix_web::web::get().to_async(resources::images::get_one::route)
                )
        })
            .bind(address)
            .unwrap()
            .start();

        let _ = tx.send(addr);
        let _ = sys.run();

    });
    let addr = rx.recv().unwrap();

    let _ = actix_rt::System::new("images_get_one").block_on(futures::lazy(|| {
        actix_web::client::Client::default()
            .get(format!("http://{}/images/green_0005.jpg", address))
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

                        assert_eq!(body_bytes.len(), 25897);

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
