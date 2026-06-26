use std::path::Path;

pub fn read_text_file(path: &Path) -> std::io::Result<String> {
    std::fs::read_to_string(path)
}

pub fn write_text_file(path: &Path, content: &str) -> std::io::Result<()> {
    std::fs::write(path, content)
}
