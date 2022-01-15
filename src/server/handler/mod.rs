mod media;

use actix_web::{get, HttpResponse, web};
use super::AppState;
pub use media::*;

pub mod response {
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct PhotoList {
        pub photos: Vec<String>,
    }
}

use response::*;

/// サムネを持つ画像ファイルのリストを取得する
#[get("/photos/list")]
async fn get_photo_list(state: web::Data<AppState>) -> HttpResponse {
    use crate::thumbs::{get_image_filenames, get_origin_filename};

    let filenames = get_image_filenames(&state.thumbs_dir)
        .unwrap_or(vec![]);

    let filenames = filenames.into_iter()
        .flat_map(|s | get_origin_filename(&s))
        .collect::<Vec<String>>();

    let r = PhotoList {
        photos: filenames,
    };

    HttpResponse::Ok().json(r)
}

#[get("/photo/thumb/{name}")]
pub async fn get_thumb(path: web::Path<String>, state: web::Data<AppState>) -> HttpResponse {
    use std::fs::File;
    use std::io::Read;
    use crate::thumbs::get_thumb_filename;

    let name = state.thumbs_dir.join(get_thumb_filename(&path.into_inner()));

    let mut file = match File::open(name) {
        Err(_) => return HttpResponse::NotFound().body(""),
        Ok(file) => file,
    };

    let mut buf: Vec<u8> = vec![];
    let _ = file.read_to_end(&mut buf);

    HttpResponse::Ok()
        .content_type("image/jpeg")
        .body(buf)
}

#[get("/photo/{name}")]
pub async fn get_photo(path: web::Path<String>, state: web::Data<AppState>) -> HttpResponse {
    use std::fs::File;
    use std::io::Read;
    
    let name = state.images_dir.join(&path.into_inner());
    let mut file = match File::open(name) {
        Err(_) => return HttpResponse::NotFound().body(""),
        Ok(file) => file,
    };

    let mut buf: Vec<u8> = vec![];
    let _ = file.read_to_end(&mut buf);

    HttpResponse::Ok()
        .content_type("image/jpeg")
        .body(buf)
}
