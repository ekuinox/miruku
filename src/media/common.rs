use std::path::{Path, PathBuf};

/// 対象の拡張子
const TARGET_EXTENSIONS: [&str; 2] = ["jpeg", "jpg"];

// メディアのディレクトリ
pub const MEDIA_DIRECTORY_NAME: &str = "media";

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
                vec![path]
            }
        })
        .collect()
}

/// 対象のファイルかチェックする
pub fn is_target(path: &Path) -> bool {
    path.extension()
        .map(|ext| {
            let ext = ext.to_string_lossy().to_string().to_lowercase();
            TARGET_EXTENSIONS.contains(&ext.as_str())
        })
        .unwrap_or(false)
}

/// 画像(jpeg)ファイルのみをリストで取得する
/// フルパスで取得する
pub fn get_image_filenames(dir: &Path) -> Vec<PathBuf> {
    // 最大5階層まで検索する
    const RECURSIVE_DEPTH: u32 = 5;

    let entries = get_filenames_recursive(dir, RECURSIVE_DEPTH);

    

    entries
        .into_iter()
        .flat_map(|path| {
            if is_target(&path) {
                path.canonicalize().ok()
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}
