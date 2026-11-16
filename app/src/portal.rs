use std::path::PathBuf;

use ashpd::desktop::file_chooser::{FileFilter, SelectedFiles};
use sabine::documents::{Handed, within_reach};

/// Turns a `file://` URI into a path, decoding its escaped bytes.
fn file_path(uri: &str) -> Option<PathBuf> {
    let encoded = uri.strip_prefix("file://")?.as_bytes();
    let mut bytes = Vec::with_capacity(encoded.len());
    let mut index = 0;
    while index < encoded.len() {
        let hex = encoded
            .get(index + 1..index + 3)
            .and_then(|hex| std::str::from_utf8(hex).ok());
        match (
            encoded[index],
            hex.and_then(|hex| u8::from_str_radix(hex, 16).ok()),
        ) {
            (b'%', Some(byte)) => {
                bytes.push(byte);
                index += 3;
            }
            (byte, _) => {
                bytes.push(byte);
                index += 1;
            }
        }
    }
    Some(PathBuf::from(
        <std::ffi::OsString as std::os::unix::ffi::OsStringExt>::from_vec(bytes),
    ))
}

fn first_path(files: SelectedFiles) -> Option<PathBuf> {
    files.uris().first().and_then(|uri| file_path(uri.as_str()))
}

async fn reach(path: PathBuf, handed: Handed) -> Option<PathBuf> {
    within_reach(vec![path], handed).await.ok()?.pop()
}

/// Asks the desktop for a folder. One the app already reaches keeps its path, so the library can
/// watch it; any other comes through the document portal.
pub async fn pick_folder(title: &str) -> Option<PathBuf> {
    let request = SelectedFiles::open_file()
        .title(title)
        .modal(true)
        .directory(true)
        .send()
        .await
        .ok()?;
    let folder = first_path(request.response().ok()?)?;
    if std::fs::read_dir(&folder).is_ok() {
        return Some(folder);
    }
    reach(folder, Handed::Existing).await
}

/// Asks the desktop where to save a file.
pub async fn pick_save(title: &str, name: &str, filter: (&str, &str)) -> Option<PathBuf> {
    let request = SelectedFiles::save_file()
        .title(title)
        .modal(true)
        .current_name(name)
        .filter(FileFilter::new(filter.0).glob(filter.1))
        .send()
        .await
        .ok()?;
    reach(first_path(request.response().ok()?)?, Handed::New).await
}
