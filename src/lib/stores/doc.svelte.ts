class DocStore {
  text = $state("");
  path = $state<string | null>(null);
  savedText = $state("");

  get dirty() {
    return this.text !== this.savedText;
  }

  setText(t: string) {
    this.text = t;
  }

  load(path: string, text: string) {
    this.path = path;
    this.text = text;
    this.savedText = text;
  }

  markSaved() {
    this.savedText = this.text;
  }
}

export const doc = new DocStore();
