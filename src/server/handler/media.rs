use crate::server::AppState;
use actix_web::{get, web, HttpResponse};

pub mod response {
    use crate::media::MediaId;
    use serde::Serialize;
    use std::collections::HashMap;

    #[derive(Serialize)]
    pub struct Meta {
        pub id: String,
        pub origin_name: String,
        pub date: String,
        pub attributes: Option<HashMap<String, String>>,
    }

    #[derive(Serialize)]
    pub struct MediaIds {
        pub ids: Vec<MediaId>,
    }
}

/// メディアIDの一覧を取得するAPI
#[get("/media/ids")]
pub async fn get_media_ids(state: web::Data<AppState>) -> HttpResponse {
    use crate::media::*;
    use response::MediaIds;

    let mut conn = match create_connection(&state.data_dir).await {
        Ok(conn) => conn,
        Err(err) => {
            eprintln!("{:?}", err);
            return HttpResponse::InternalServerError().body("");
        }
    };

    match MediaMeta::ids(&mut conn).await {
        Ok(ids) => {
            let response = MediaIds { ids };
            HttpResponse::Ok().json(response)
        }
        Err(err) => {
            eprintln!("{:?}", err);
            return HttpResponse::InternalServerError().body("");
        }
    }
}

/// メディアのサムネイルを取得するAPI
#[get("/media/thumb/{media_id}")]
pub async fn get_media_thumb(path: web::Path<String>, state: web::Data<AppState>) -> HttpResponse {
    use crate::media::*;

    let mut conn = match create_connection(&state.data_dir).await {
        Ok(conn) => conn,
        Err(err) => {
            eprintln!("{:?}", err);
            return HttpResponse::InternalServerError().body("");
        }
    };

    let meta = match MediaMeta::open(&mut conn, &path.into_inner()).await {
        Ok(meta) => meta,
        Err(_) => return HttpResponse::NotFound().body(""),
    };
    let media: Media = meta.into();

    let thumb_buf = match media.get_thumb(&state.data_dir).await {
        Ok(buf) => buf,
        Err(_) => return HttpResponse::InternalServerError().body(""),
    };

    HttpResponse::Ok()
        .content_type("image/jpeg")
        .body(thumb_buf)
}

/// メディアのオリジナルデータを取得するAPI
#[get("/media/origin/{media_id}")]
pub async fn get_media_origin(path: web::Path<String>, state: web::Data<AppState>) -> HttpResponse {
    use crate::media::*;

    let mut conn = match create_connection(&state.data_dir).await {
        Ok(conn) => conn,
        Err(err) => {
            eprintln!("{:?}", err);
            return HttpResponse::InternalServerError().body("");
        }
    };

    let meta = match MediaMeta::open(&mut conn, &path.into_inner()).await {
        Ok(meta) => meta,
        Err(_) => return HttpResponse::NotFound().body(""),
    };
    let media: Media = meta.into();

    let thumb_buf = match media.get_origin(&state.data_dir).await {
        Ok(buf) => buf,
        Err(_) => return HttpResponse::InternalServerError().body(""),
    };

    HttpResponse::Ok()
        .content_type("image/jpeg") // とりあえず
        .body(thumb_buf)
}

/// メタ情報を取得する
#[get("/media/meta/{media_id}")]
pub async fn get_media_meta(path: web::Path<String>, state: web::Data<AppState>) -> HttpResponse {
    use crate::media::*;
    use response::Meta;
    use std::ops::Deref;

    let mut conn = match create_connection(&state.data_dir).await {
        Ok(conn) => conn,
        Err(err) => {
            eprintln!("{:?}", err);
            return HttpResponse::InternalServerError().body("");
        }
    };

    let meta = match MediaMeta::open(&mut conn, &path.into_inner()).await {
        Ok(meta) => meta,
        Err(_) => return HttpResponse::NotFound().body(""),
    };

    let response = Meta {
        id: meta.media_id.deref().clone(),
        origin_name: meta.origin,
        date: meta.date.map(|date| date.to_string()).unwrap_or_default(),
        attributes: meta.attributes.map(|json| json.0),
    };

    HttpResponse::Ok().json(response)
}
