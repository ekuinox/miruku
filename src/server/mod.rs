mod handler;

use std::path::Path;
use actix_web::{HttpServer, App};
use anyhow::Result;
use handler::*;

#[derive(Debug)]
pub struct Server<'a> {
    pub images_dir: &'a Path,
    pub thumbs_dir: &'a Path,
    pub port: u64,
}

impl <'a> Server<'a> {
    pub async fn start(&self) -> Result<()> {
        let bind_to = format!("0.0.0.0:{}", self.port);
        let _ = HttpServer::new(|| App::new()
            .service(get_photo_list)
        )
            .bind(&bind_to)?
            .run()
            .await?;
        Ok(())
    }
}
