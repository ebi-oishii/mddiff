<script lang="ts">
  import { getVersion } from "@tauri-apps/api/app";

  let version = $state<string>("");
  $effect(() => {
    getVersion().then((v) => (version = v));
  });
</script>

<main>
  <h1>mdv</h1>
  <p>Lightweight Markdown viewer and editor.</p>
  <p class="meta">Phase 0 scaffold · v{version}</p>
</main>

<style>
  :global(:root) {
    color-scheme: light dark;
    font-family: system-ui, -apple-system, "Segoe UI", Roboto, sans-serif;
  }
  main {
    max-width: 640px;
    margin: 4rem auto;
    padding: 0 1.5rem;
  }
  h1 {
    margin: 0 0 0.5rem;
    font-size: 2.5rem;
    letter-spacing: -0.02em;
  }
  p {
    margin: 0.25rem 0;
    color: light-dark(#444, #bbb);
  }
  .meta {
    font-size: 0.875rem;
    color: light-dark(#888, #888);
  }
</style>
