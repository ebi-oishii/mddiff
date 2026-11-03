import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { test } from "node:test";

test("email-heavy Markdown finishes parsing without quadratic linkification", () => {
  // Use a child so a vulnerable synchronous parser cannot hang the test runner.
  // 560 KB is well below the editor's large-file warning. This exercises the
  // formerly quadratic path with a generous limit for slower CI machines.
  const result = spawnSync(process.execPath, ["--input-type=module", "-e", `
    import MarkdownIt from "markdown-it";
    const md = new MarkdownIt({ html: true, linkify: true, breaks: false, typographer: true });
    const html = md.render("a@b.co\\n".repeat(80000));
    if ((html.match(/mailto:/g) ?? []).length !== 80000) process.exit(1);
  `], { cwd: new URL("..", import.meta.url), timeout: 15000, encoding: "utf8" });
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stderr);
});
