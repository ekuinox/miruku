use once_cell::sync::Lazy;
use regex::Regex;
use std::path::{Path, PathBuf};

// JPEG画像をフィルタするための正規表現
static JPEG_FILE_EXT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\.jpe?g$").expect("JPEG_FILE_EXT_REGEX is not valid"));

// メディアのディレクトリ
pub const MEDIA_DIRECTORY_NAME: &'static str = "media";

/// path のファイルを再起的に全て取得する
pub fn get_filenames_recursive(path: &Path, depth: u32) -> Vec<PathBuf> {
    use std::fs::*;

    if depth == 0 {
        return vec![];
    }

    let entries = match read_dir(path) {
        Ok(entries) => entries,
        Err(_) => return vec![],
    };

    entries
        .into_iter()
        .flatten()
        .flat_map(|entry| {
            let path = entry.path();
            if path.is_dir() {
                get_filenames_recursive(&path, depth - 1)
            } else {
                vec![path.to_owned()]
            }
        })
        .collect()
}

/// 画像(jpeg)ファイルのみをリストで取得する
pub fn get_image_filenames(dir: &Path) -> Vec<String> {
    // 最大5階層まで検索する
    const RECURSIVE_DEPTH: u32 = 5;

    let entries = get_filenames_recursive(dir, RECURSIVE_DEPTH);

    let entries = entries
        .into_iter()
        .map(|path| path.into_os_string().to_string_lossy().to_string())
        .collect::<Vec<_>>();

    // jpegファイルでフィルタ
    let entries = entries
        .into_iter()
        .filter(|s| JPEG_FILE_EXT_REGEX.is_match(&s.to_lowercase()))
        .collect::<Vec<String>>();

    entries
}
