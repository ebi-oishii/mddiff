//! Native menu bar (Mac: system bar, Win/Linux: window top bar). Each item
//! emits a `menu-event` to the webview with its string id; the frontend
//! dispatches that id to the same handlers that the in-app ☰ menu uses.
//!
//! Mobile (`cfg(target_os = "android" | "ios")`) is excluded — set_menu has
//! no effect there and the in-app ☰ menu is the only path.

use tauri::menu::{AboutMetadata, Menu, MenuItem, PredefinedMenuItem, SubmenuBuilder};
use tauri::{AppHandle, Wry};

pub fn build(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let about = AboutMetadata {
        name: Some("mddiff".into()),
        version: Some(env!("CARGO_PKG_VERSION").into()),
        ..Default::default()
    };

    let app_menu = SubmenuBuilder::new(app, "mddiff")
        .about(Some(about))
        .separator()
        .item(&MenuItem::with_id(
            app,
            "preferences",
            "Settings…",
            true,
            Some("CmdOrCtrl+,"),
        )?)
        .separator()
        .item(&PredefinedMenuItem::hide(app, None)?)
        .item(&PredefinedMenuItem::hide_others(app, None)?)
        .item(&PredefinedMenuItem::show_all(app, None)?)
        .separator()
        .item(&PredefinedMenuItem::quit(app, None)?)
        .build()?;

    let export_submenu = SubmenuBuilder::new(app, "Export")
        .item(&MenuItem::with_id(
            app,
            "export_html",
            "HTML",
            true,
            Some("CmdOrCtrl+Shift+H"),
        )?)
        .item(&MenuItem::with_id(
            app,
            "export_pdf",
            "PDF…",
            true,
            Some("CmdOrCtrl+Shift+P"),
        )?)
        .item(&MenuItem::with_id(
            app,
            "export_text",
            "Plain text",
            true,
            Some("CmdOrCtrl+Shift+T"),
        )?)
        .item(&MenuItem::with_id(
            app,
            "export_docx",
            "DOCX",
            true,
            Some("CmdOrCtrl+Shift+D"),
        )?)
        .item(&MenuItem::with_id(
            app,
            "export_mddiff",
            ".mddiff (with history)",
            true,
            Some("CmdOrCtrl+Shift+M"),
        )?)
        .build()?;

    let file_menu = SubmenuBuilder::new(app, "File")
        .item(&MenuItem::with_id(
            app,
            "open",
            "Open…",
            true,
            Some("CmdOrCtrl+O"),
        )?)
        .separator()
        .item(&MenuItem::with_id(
            app,
            "save",
            "Save",
            true,
            Some("CmdOrCtrl+S"),
        )?)
        .item(&MenuItem::with_id(
            app,
            "save_as",
            "Save As…",
            true,
            Some("CmdOrCtrl+Shift+S"),
        )?)
        .item(&MenuItem::with_id(
            app,
            "reload",
            "Reload from Disk",
            true,
            Some("CmdOrCtrl+Shift+R"),
        )?)
        .separator()
        .item(&MenuItem::with_id(
            app,
            "sample",
            "Load Sample",
            true,
            None::<&str>,
        )?)
        .separator()
        .item(&export_submenu)
        .separator()
        .item(&PredefinedMenuItem::close_window(app, None)?)
        .build()?;

    let edit_menu = SubmenuBuilder::new(app, "Edit")
        .item(&PredefinedMenuItem::undo(app, None)?)
        .item(&PredefinedMenuItem::redo(app, None)?)
        .separator()
        .item(&PredefinedMenuItem::cut(app, None)?)
        .item(&PredefinedMenuItem::copy(app, None)?)
        .item(&PredefinedMenuItem::paste(app, None)?)
        .item(&PredefinedMenuItem::select_all(app, None)?)
        .build()?;

    // Grouped View menu: Live Preview / WYSIWYG / Diff are the 3 top-level
    // modes; Source / Preview are sub-modes reached via ⌘⇧1 / ⌘⇧2 (toggle).
    let view_menu = SubmenuBuilder::new(app, "View")
        .item(&MenuItem::with_id(
            app,
            "mode_live",
            "Live Preview",
            true,
            Some("CmdOrCtrl+1"),
        )?)
        .item(&MenuItem::with_id(
            app,
            "mode_source",
            "    Source",
            true,
            Some("CmdOrCtrl+Shift+1"),
        )?)
        .separator()
        .item(&MenuItem::with_id(
            app,
            "mode_wysiwyg",
            "WYSIWYG",
            true,
            Some("CmdOrCtrl+2"),
        )?)
        .item(&MenuItem::with_id(
            app,
            "mode_preview",
            "    Preview",
            true,
            Some("CmdOrCtrl+Shift+2"),
        )?)
        .separator()
        .item(&MenuItem::with_id(
            app,
            "mode_diff",
            "Diff",
            true,
            Some("CmdOrCtrl+3"),
        )?)
        .build()?;

    Menu::with_items(app, &[&app_menu, &file_menu, &edit_menu, &view_menu])
}
