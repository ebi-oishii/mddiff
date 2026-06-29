use std::io::{BufWriter, Cursor};
use std::path::{Path, PathBuf};

use chrono::Local;
use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_clipboard_manager::ClipboardExt;

#[derive(Serialize)]
pub struct PastedImage {
    /// Path written to disk, relative to the document's directory. Suitable
    /// for inserting verbatim into a Markdown link: `![](./<rel>)`.
    pub rel_path: String,
    /// Absolute path on disk (debug-only, useful for surfacing errors).
    pub abs_path: PathBuf,
}

/// Pull an image from the system clipboard, encode it as PNG, and write it
/// to a sibling `<filename>.assets/` directory next to the open document.
///
/// Filename format: `YYYY-MM-DD-HHMMSS.png`. If a file with that name already
/// exists (multiple pastes in the same second) a `-N` counter is appended.
///
/// Returns `Ok(None)` when the clipboard contains no image (vs. some other
/// error). The frontend uses this to decide whether to fall through to the
/// editor's normal paste behavior (text).
#[tauri::command]
pub async fn paste_image(
    app: AppHandle,
    doc_path: PathBuf,
) -> Result<Option<PastedImage>, String> {
    // 1. Read image from clipboard. Plugin returns a friendly error if there
    //    isn't one; treat that as "no image" rather than a hard failure so
    //    plain-text pastes still work in the host's normal flow.
    let image = match app.clipboard().read_image() {
        Ok(img) => img,
        Err(_) => return Ok(None),
    };
    let width = image.width();
    let height = image.height();
    let rgba = image.rgba();

    // 2. Compute the asset directory. Pattern matches Typora's default:
    //    `notes/file.md` → `notes/file.assets/`. Using the stem (no ext)
    //    means swapping a .md → .markdown rename doesn't orphan assets.
    let doc_dir = doc_path
        .parent()
        .ok_or_else(|| "document has no parent directory".to_string())?;
    let stem = doc_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "document name has no valid stem".to_string())?;
    let asset_dir = doc_dir.join(format!("{stem}.assets"));
    std::fs::create_dir_all(&asset_dir)
        .map_err(|e| format!("failed to create asset directory: {e}"))?;

    // 3. Generate a timestamp filename, with a `-N` counter on collision so
    //    rapid pastes (same second) don't overwrite each other.
    let stamp = Local::now().format("%Y-%m-%d-%H%M%S").to_string();
    let (abs_path, file_name) = next_available(&asset_dir, &stamp);

    // 4. Encode RGBA → PNG and write.
    let png_bytes = encode_png(rgba, width, height)
        .map_err(|e| format!("PNG encode failed: {e}"))?;

    // The watcher only watches the .md document, not the .assets/ subdir, so
    // writing the image here can't trip a self-write loop.
    std::fs::write(&abs_path, &png_bytes)
        .map_err(|e| format!("failed to write image: {e}"))?;

    // 5. Relative path the frontend can insert verbatim. Use forward slashes
    //    on Windows too — Markdown link paths shouldn't have backslashes.
    let rel_path = format!("{stem}.assets/{file_name}");

    Ok(Some(PastedImage { rel_path, abs_path }))
}

/// Find the first non-conflicting filename in `dir` for the given timestamp:
/// `2026-06-28-152233.png`, `…-1.png`, `…-2.png`, …
fn next_available(dir: &Path, stamp: &str) -> (PathBuf, String) {
    let first = format!("{stamp}.png");
    let candidate = dir.join(&first);
    if !candidate.exists() {
        return (candidate, first);
    }
    for n in 1..1000 {
        let name = format!("{stamp}-{n}.png");
        let candidate = dir.join(&name);
        if !candidate.exists() {
            return (candidate, name);
        }
    }
    // Fall back to a uniquified epoch suffix on the absurd chance every slot
    // is taken. Shouldn't happen in practice.
    let now = chrono::Utc::now().timestamp_micros();
    let name = format!("{stamp}-{now}.png");
    (dir.join(&name), name)
}

fn encode_png(rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>, png::EncodingError> {
    let mut buf: Vec<u8> = Vec::with_capacity(rgba.len() / 2);
    {
        let writer = BufWriter::new(Cursor::new(&mut buf));
        let mut encoder = png::Encoder::new(writer, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(rgba)?;
    }
    Ok(buf)
}
