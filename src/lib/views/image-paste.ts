import { EditorView } from "@codemirror/view";
import { pasteImage } from "$lib/ipc/clipboard";
import { doc } from "$lib/stores/doc.svelte";
import { i18n } from "$lib/i18n/index.svelte";

/**
 * Callback invoked when the image-paste flow fails. Lets the host surface
 * the error via its normal banner instead of getting swallowed.
 */
export type ImagePasteErrorHandler = (message: string) => void;

/**
 * CodeMirror DOM-event extension that intercepts clipboard paste, saves any
 * image content to `<doc-stem>.assets/`, and inserts a Markdown image link
 * at the cursor.
 *
 * Behavior:
 *   - Plain text paste: fall through to CodeMirror's default (no-op here).
 *   - Image in clipboard but no file open: notify the host via `onError`
 *     because we have nowhere to save the asset.
 *   - Image in clipboard, file open: preventDefault, save asynchronously,
 *     then dispatch the `![](rel)` insertion at the captured cursor.
 *
 * We rely on the Tauri side to detect whether the clipboard actually has an
 * image (more reliable across OS than `event.clipboardData.types` which the
 * webview sometimes hides for native screenshot tools). The browser hint is
 * still used as a fast pre-filter so non-image pastes don't make a needless
 * IPC round-trip.
 */
export function imagePaste(onError: ImagePasteErrorHandler) {
  return EditorView.domEventHandlers({
    paste(event, view) {
      const clipData = event.clipboardData;
      if (!clipData) return false;

      // Cheap pre-filter: skip the IPC roundtrip when the webview already
      // sees nothing image-like. Note `types` includes `Files` for drag-n-
      // drop and some clipboard variants.
      const types = Array.from(clipData.types);
      const looksLikeImage =
        types.some((t) => t.startsWith("image/")) || types.includes("Files");
      if (!looksLikeImage) return false;

      const docPath = doc.path;
      if (!docPath) {
        // Block the paste with a hint — the alternative (silent text-paste
        // fall-through) leaves the user confused why the image vanished.
        event.preventDefault();
        onError(i18n.t("errors.saveFirstForImage"));
        return true;
      }

      // Capture the selection range right now so any view churn during the
      // async save doesn't move the insert point.
      const { from, to } = view.state.selection.main;
      event.preventDefault();

      pasteImage(docPath)
        .then((result) => {
          if (!result) return; // wasn't an image after all
          const link = `![](${result.rel_path})`;
          view.dispatch({
            changes: { from, to, insert: link },
            selection: { anchor: from + link.length },
          });
        })
        .catch((e) => {
          onError(String(e));
        });

      return true;
    },
  });
}
