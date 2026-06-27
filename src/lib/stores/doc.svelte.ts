class DocStore {
  text = $state("");
  path = $state<string | null>(null);
  savedText = $state("");
  gitAvailable = $state(false);

  get dirty() {
    return this.text !== this.savedText;
  }

  setText(t: string) {
    this.text = t;
  }

  load(path: string | null, text: string, gitAvailable: boolean) {
    this.path = path;
    this.text = text;
    this.savedText = text;
    this.gitAvailable = gitAvailable;
  }

  markSaved() {
    this.savedText = this.text;
  }
}

export const doc = new DocStore();
