use actix_web::{get, HttpResponse};

pub mod response {
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct PhotoList {
        pub photos: Vec<String>,
    }
}

use response::*;

#[get("/photo/list")]
async fn get_photo_list() -> HttpResponse {
    let r = PhotoList {
        photos: vec![],
    };
    HttpResponse::Ok().json(r)
}
