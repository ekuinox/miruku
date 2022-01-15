use std::ops::Deref;

use actix_web::{get, HttpResponse, web};
use crate::server::AppState;

pub mod response {
    use std::collections::HashMap;

    use serde::Serialize;

    use crate::media::MediaId;

    #[derive(Serialize)]
    pub struct Meta {
        pub id: String,
        pub origin_name: String,
        pub attributes: HashMap<String, String>,
    }

    #[derive(Serialize)]
    pub struct MediaIds {
        pub ids: Vec::<MediaId>,
    }
}

/// メディアIDの一覧を取得するAPI
#[get("/media/ids")]
pub async fn get_media_ids(state: web::Data<AppState>) -> HttpResponse {
    use crate::media::*;
    use response::MediaIds;
    let data_directory = &state.data_dir.join(common::MEDIA_DIRECTORY_NAME);

    match MediaMeta::ids(data_directory).await {
        Ok(ids) => {
            let response = MediaIds {
                ids,
            };
            HttpResponse::Ok().json(response)
        },
        Err(_) => HttpResponse::InternalServerError().body(""),
    }
}

/// メディアのサムネイルを取得するAPI
#[get("/media/thumb/{media_id}")]
pub async fn get_media_thumb(
    path: web::Path<String>,
    state: web::Data<AppState>
) -> HttpResponse {
    use crate::media::*;
    let data_directory = &state.data_dir.join(common::MEDIA_DIRECTORY_NAME);

    let meta = match MediaMeta::open(data_directory, &path.into_inner()).await {
        Ok(meta) => meta,
        Err(_) => return HttpResponse::NotFound().body(""),
    };
    let media: Media = meta.into();

    let thumb_buf = match media.get_thumb(data_directory).await {
        Ok(buf) => buf,
        Err(_) => return HttpResponse::InternalServerError().body(""),
    };

    HttpResponse::Ok()
        .content_type("image/jpeg")
        .body(thumb_buf)
}

/// メディアのオリジナルデータを取得するAPI
#[get("/media/origin/{media_id}")]
pub async fn get_media_origin(
    path: web::Path<String>,
    state: web::Data<AppState>
) -> HttpResponse {
    use crate::media::*;
    let data_directory = &state.data_dir.join(common::MEDIA_DIRECTORY_NAME);

    let meta = match MediaMeta::open(data_directory, &path.into_inner()).await {
        Ok(meta) => meta,
        Err(_) => return HttpResponse::NotFound().body(""),
    };
    let media: Media = meta.into();

    let thumb_buf = match media.get_origin(data_directory).await {
        Ok(buf) => buf,
        Err(_) => return HttpResponse::InternalServerError().body(""),
    };

    HttpResponse::Ok()
        .content_type("image/jpeg") // とりあえず
        .body(thumb_buf)
}

/// メタ情報を取得する
#[get("/media/meta/{media_id}")]
pub async fn get_media_meta(
    path: web::Path<String>,
    state: web::Data<AppState>
) -> HttpResponse {
    use crate::media::*;
    use response::Meta;

    let data_directory = &state.data_dir.join(common::MEDIA_DIRECTORY_NAME);

    let meta = match MediaMeta::open(data_directory, &path.into_inner()).await {
        Ok(meta) => meta,
        Err(_) => return HttpResponse::NotFound().body(""),
    };

    let response = Meta {
        id: meta.id.deref().clone(),
        origin_name: meta.origin,
        attributes: meta.attributes,
    };

    HttpResponse::Ok().json(response)
}
