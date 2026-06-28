/**
 * Turn a raw error (typically a stringified Rust error coming back through a
 * Tauri command) into a sentence the user can act on. The Rust side just does
 * `e.to_string()`, so we see things like `Os { code: 13, kind:
 * PermissionDenied, message: "Permission denied" }` — that's not something
 * to put in a banner.
 *
 * Pass `op` to bias the wording for the operation that was running so
 * "Permission denied" can become "Cannot write" or "Cannot read" depending
 * on context.
 */
export type Op = "read" | "write" | "other";

export function humanizeError(e: unknown, op: Op = "other"): string {
  const raw = String(e);

  // mdv-core large-file errors are already human-friendly, just pass through.
  if (raw.includes("exceeds the") && raw.includes("warning threshold")) {
    return raw;
  }
  if (raw.includes("exceeds the") && raw.includes("hard limit")) {
    return "This file is too large to open (over 100 MB).";
  }

  // Rust std::io::Error kinds — match by name so we work across formatting
  // variants ("kind: PermissionDenied" or just "PermissionDenied" in
  // anyhow output).
  if (/PermissionDenied|Permission denied/.test(raw)) {
    return op === "write"
      ? "Permission denied — the file can't be written. Check that you can write to this location."
      : "Permission denied — the file can't be read.";
  }
  if (/NotFound|No such file/.test(raw)) {
    return op === "write"
      ? "The destination directory doesn't exist."
      : "File not found. It may have been moved or deleted.";
  }
  if (/AlreadyExists|File exists/.test(raw)) {
    return "A file with that name already exists at the destination.";
  }
  if (/NoSpace|No space|No room left/.test(raw)) {
    return "The disk is full.";
  }
  if (/ReadOnlyFilesystem|Read-only file system/.test(raw)) {
    return "The destination is on a read-only file system.";
  }
  if (/InvalidData|stream did not contain valid UTF-8/.test(raw)) {
    return "This file isn't valid UTF-8 text. mdv only supports text files.";
  }
  if (/Interrupted/.test(raw)) {
    return "The operation was interrupted. Try again.";
  }

  // Last resort: extract the embedded OS `message: "…"` if present.
  const osMessage = raw.match(/message:\s*"([^"]+)"/);
  if (osMessage) {
    return capitalize(osMessage[1]);
  }

  // mdv-core "io error: …" wrapper — recurse on the inner message.
  const ioWrapped = raw.match(/^io error:\s*(.+)$/);
  if (ioWrapped) return humanizeError(ioWrapped[1], op);

  return raw;
}

function capitalize(s: string): string {
  return s.charAt(0).toUpperCase() + s.slice(1);
}
