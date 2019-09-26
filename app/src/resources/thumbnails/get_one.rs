use futures::{Future};
use actix_web::{
    self,
    HttpRequest,
    HttpResponse,
};
use utils;

#[allow(dead_code)]
pub fn route(req: HttpRequest) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    utils::app::open_file_to_response(
        utils::app::get_thumbnails_dir()
            .join(
                req
                    .match_info()
                    .query("filename")
                    .parse::<std::path::PathBuf>()
                    .unwrap()
            )
    )
}
