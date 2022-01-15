use anyhow::Result;
use std::path::Path;

// リサイズ後の画像サイズ
pub const TARGET_SIZE: usize = 480;

// リサイズする際のフィルタ種別 https://docs.rs/image/latest/image/imageops/enum.FilterType.html
pub const IMAGE_FILTER_TYPE: image::imageops::FilterType = image::imageops::FilterType::Nearest;

/// リサイズして保存する
pub fn create_thumb(source: &Path, dest: &Path) -> Result<()> {
    use image::GenericImageView;
    let img = image::open(source)?;

    let width = img.width() as usize;
    let height = img.height() as usize;    

    if width > TARGET_SIZE || height > TARGET_SIZE {
        let (target_width, target_height) = 
            if width > height {
                let ratio: f32 = TARGET_SIZE as f32 / width as f32;
                (TARGET_SIZE, (height as f32 * ratio) as usize)
            } else {
                let ratio: f32 = TARGET_SIZE as f32 / height as f32;
                ((width as f32 * ratio) as usize, TARGET_SIZE)
            };
        let resized_img = img.resize(
            target_width as u32,
            target_height as u32,
            IMAGE_FILTER_TYPE
        );
        let _ = resized_img.save(dest)?;
        Ok(())
    } else {
        let _ = img.save(dest)?;
        Ok(())
    }
}
