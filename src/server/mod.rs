mod handler;

use std::path::{Path, PathBuf};
use actix_web::{HttpServer, App, web};
use anyhow::Result;
use handler::*;

#[derive(Debug, Clone)]
pub struct Server<'a> {
    pub data_dir: &'a Path,
    pub port: u64,
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub data_dir: PathBuf,
}

impl <'a> Server<'a> {
    pub async fn start(&self) -> Result<()> {
        let bind_to = format!("0.0.0.0:{}", self.port);
        let state = AppState {
            data_dir: self.data_dir.to_owned(),
        };
        let _ = HttpServer::new(move || App::new()
            .app_data(web::Data::new(state.clone()))
            .service(get_media_ids)
            .service(get_media_meta)
            .service(get_media_origin)
            .service(get_media_thumb)
        )
            .bind(&bind_to)?
            .run()
            .await?;
        Ok(())
    }
}
