use anyhow::Result;
use regex::Regex;
use once_cell::sync::Lazy;
use std::path::Path;
use rayon::prelude::*;

// リサイズ後の画像サイズ
const TARGET_SIZE: usize = 480;

// リサイズする際のフィルタ種別 https://docs.rs/image/latest/image/imageops/enum.FilterType.html
const IMAGE_FILTER_TYPE: image::imageops::FilterType = image::imageops::FilterType::Nearest;

/// サムネ画像ファイル用サフィックス
const THUMB_IMAGE_SUFFIX: &'static str = ".thumb.jpg";

// JPEG画像をフィルタするための正規表現
static JPEG_FILE_EXT_REGEX: Lazy<Regex> = Lazy::new(
    || Regex::new(r"\.jpe?g$").expect("JPEG_FILE_EXT_REGEX is not valid")
);

/// サムネ画像用のファイル名を取得する
pub fn get_thumb_filename(name: &String) -> String {
    format!("{}{}", name, THUMB_IMAGE_SUFFIX)
}

/// サムネ用画像のファイル名から元画像のファイル名を取得する
pub fn get_origin_filename(name: &String) -> Option<String> {
    name.strip_suffix(".thumb.jpg").map(|s| s.to_string())
}

/// リサイズして保存する
fn resize(source: &Path, dest: &Path) -> Result<()> {
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

/// dir 下のファイル名をStringのリストで取得する
fn get_filenames(dir: &Path) -> Result<Vec<String>> {
    use std::fs::*;

    let entries = read_dir(dir)?;

    // ファイル一覧をStringで取得
    let entries = entries.into_iter()
        .flat_map(|entry| {
            if let Ok(entry) = entry {
                entry.file_name().to_str().map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect::<Vec<String>>();
    Ok(entries)
}

/// 画像(jpeg)ファイルのみをリストで取得する
pub fn get_image_filenames(dir: &Path) -> Result<Vec<String>> {
    let entries = get_filenames(dir)?;

    // jpegファイルでフィルタ
    let entries = entries.into_iter()
        .filter(|s| JPEG_FILE_EXT_REGEX.is_match(&s.to_lowercase()))
        .collect::<Vec<String>>();

    Ok(entries)
}

/// サムネイル画像を作成して保存する
pub fn create_thumbs(source: &Path, dest: &Path) -> Result<()> {
    // source/(*.jpe?g) を dest/(1).thumb.jpg として保存する

    // dest のファイル一覧を取得
    let dest_entries = get_image_filenames(dest)?;
    
    // source のファイル一覧を取得
    let entries = get_image_filenames(source)?;

    // リサイズしていく
    let _ = entries.par_iter().try_for_each(|entry| -> Result<()> {
        let thumb_name = get_thumb_filename(&entry);
        let existed = dest_entries.iter().any(|s| *s == thumb_name);
        println!("entry = {}, existed = {}", entry, existed);

        // 既に dest にファイルが作成されている場合はスキップする
        if existed {
            return Ok(());
        }

        let source = source.join(&entry);
        let dest = dest.join(&thumb_name);
        let _ = resize(source.as_path(), dest.as_path())?;
        Ok(())
    })?;

    Ok(())
}
