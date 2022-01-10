mod handler;

use std::path::{Path, PathBuf};
use actix_web::{HttpServer, App, web};
use anyhow::Result;
use handler::*;

#[derive(Debug, Clone)]
pub struct Server<'a> {
    pub images_dir: &'a Path,
    pub thumbs_dir: &'a Path,
    pub port: u64,
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub images_dir: PathBuf,
    pub thumbs_dir: PathBuf,
}

impl <'a> Server<'a> {
    pub async fn start(&self) -> Result<()> {
        let bind_to = format!("0.0.0.0:{}", self.port);
        let state = AppState {
            images_dir: self.images_dir.to_owned(),
            thumbs_dir: self.thumbs_dir.to_owned(),
        };
        let _ = HttpServer::new(move || App::new()
            .app_data(web::Data::new(state.clone()))
            .service(get_photo_list)
            .service(get_thumb)
            .service(get_photo)
        )
            .bind(&bind_to)?
            .run()
            .await?;
        Ok(())
    }
}
