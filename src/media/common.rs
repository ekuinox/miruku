use anyhow::Result;
use regex::Regex;
use std::path::Path;
use once_cell::sync::Lazy;

// JPEG画像をフィルタするための正規表現
static JPEG_FILE_EXT_REGEX: Lazy<Regex> = Lazy::new(
    || Regex::new(r"\.jpe?g$").expect("JPEG_FILE_EXT_REGEX is not valid")
);

// メディアのディレクトリ
pub const MEDIA_DIRECTORY_NAME: &'static str = "media";

/// dir 下のファイル名をStringのリストで取得する
pub fn get_filenames(dir: &Path) -> Result<Vec<String>> {
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
