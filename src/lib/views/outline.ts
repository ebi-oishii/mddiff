import { createLineMapMd } from "./markdown-render";

export type Heading = {
  /** 1–6, matching ATX `#`–`######`. */
  level: number;
  /** Plain-text content of the heading line. */
  text: string;
  /** 1-based source line number where the heading starts. */
  line: number;
};

// Single shared parser instance — outline extraction is a hot path (re-runs
// on every doc.text change). createLineMapMd() gives us a bare instance
// with no taskLists / typographer plugins that would shift token positions.
const md = createLineMapMd();

/**
 * Parse `text` with markdown-it and pull out every ATX / Setext heading as a
 * `{ level, text, line }` triple. Used by the outline sidebar; the renderer
 * already produces matching slugs via markdown-it-anchor, so the same
 * heading set drives both navigation and anchor links.
 */
export function extractHeadings(text: string): Heading[] {
  const tokens = md.parse(text, {});
  const out: Heading[] = [];
  for (let i = 0; i < tokens.length; i++) {
    const tok = tokens[i];
    if (tok.type !== "heading_open" || !tok.tag) continue;
    const level = parseInt(tok.tag.slice(1), 10);
    if (!Number.isFinite(level) || level < 1 || level > 6) continue;
    // The inline content sits in the next token (markdown-it block-level
    // ordering: heading_open / inline / heading_close).
    const inline = tokens[i + 1];
    const headingText = inline?.content?.trim() ?? "";
    if (!headingText) continue;
    const line = (tok.map?.[0] ?? 0) + 1;
    out.push({ level, text: headingText, line });
  }
  return out;
}

/**
 * Pick the heading that's currently "in view" based on `currentLine`. Walks
 * the list and returns the last heading whose source line is at or above
 * `currentLine`. Returns `null` when the cursor is above the first heading
 * (e.g. document preamble before any heading).
 */
export function activeHeadingIndex(
  headings: Heading[],
  currentLine: number,
): number {
  let idx = -1;
  for (let i = 0; i < headings.length; i++) {
    if (headings[i].line <= currentLine) idx = i;
    else break;
  }
  return idx;
}
