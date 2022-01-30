use std::path::{Path, PathBuf};

/// 対象の拡張子
static TARGET_EXTENSIONS: [&str; 2] = ["jpeg", "jpg"];

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
        .flat_map(|path| {
            // 対象の拡張子かチェックする
            let is_target = path.extension().map(|ext| {
                let ext = ext.to_string_lossy().to_string().to_lowercase();
                TARGET_EXTENSIONS.contains(&ext.as_str())
            }).unwrap_or(false);
            if is_target {
                Some(path.into_os_string().to_string_lossy().to_string())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    entries
}
