import { i18n } from "$lib/i18n/index.svelte";

/**
 * Turn a raw error (typically a stringified Rust error coming back through a
 * Tauri command) into a sentence the user can act on, in their selected
 * locale. The Rust side just does `e.to_string()`, so we see things like
 * `Os { code: 13, kind: PermissionDenied, message: "Permission denied" }` —
 * that's not something to put in a banner.
 *
 * Pass `op` to bias the wording for the operation that was running so
 * "Permission denied" can become "can't read" or "can't write" depending on
 * context.
 */
export type Op = "read" | "write" | "other";

export function humanizeError(e: unknown, op: Op = "other"): string {
  const raw = String(e);

  // mddiff-core large-file errors are already human-friendly (English) —
  // pass through verbatim. Localizing these requires translating the
  // strings at the Rust source.
  if (raw.includes("exceeds the") && raw.includes("warning threshold")) {
    return raw;
  }
  if (raw.includes("exceeds the") && raw.includes("hard limit")) {
    return i18n.t("errors.tooLarge");
  }

  // Rust std::io::Error kinds — match by name so we work across formatting
  // variants ("kind: PermissionDenied" or just "PermissionDenied" in
  // anyhow output).
  if (/PermissionDenied|Permission denied/.test(raw)) {
    return i18n.t(
      op === "write" ? "errors.permissionWrite" : "errors.permissionRead",
    );
  }
  if (/NotFound|No such file/.test(raw)) {
    return i18n.t(
      op === "write" ? "errors.fileNotFoundWrite" : "errors.fileNotFoundRead",
    );
  }
  if (/AlreadyExists|File exists/.test(raw)) {
    return i18n.t("errors.alreadyExists");
  }
  if (/NoSpace|No space|No room left/.test(raw)) {
    return i18n.t("errors.diskFull");
  }
  if (/ReadOnlyFilesystem|Read-only file system/.test(raw)) {
    return i18n.t("errors.readOnly");
  }
  if (/InvalidData|stream did not contain valid UTF-8/.test(raw)) {
    return i18n.t("errors.notUtf8");
  }
  if (/Interrupted/.test(raw)) {
    return i18n.t("errors.interrupted");
  }

  // Last resort: extract the embedded OS `message: "…"` if present.
  const osMessage = raw.match(/message:\s*"([^"]+)"/);
  if (osMessage) {
    return capitalize(osMessage[1]);
  }

  // mddiff-core "io error: …" wrapper — recurse on the inner message.
  const ioWrapped = raw.match(/^io error:\s*(.+)$/);
  if (ioWrapped) return humanizeError(ioWrapped[1], op);

  return raw;
}

function capitalize(s: string): string {
  return s.charAt(0).toUpperCase() + s.slice(1);
}
