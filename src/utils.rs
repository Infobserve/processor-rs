use walkdir::WalkDir;

/// Recursively finds and returns the relative path
/// to all files that satisfy the `ext` extension filter
///
/// # Arguments
///
/// * `dir` - The path to the root directory that contains the Yara rulefiles
/// * `ext` - The extension to look for (no leading '.')
///
/// # Examples
///
/// ```
/// let rule_files: Vec<&str> = utils::rec_get_files_by_ext("yara-rules", "yar");
/// // Vec<&str>: ["yara-rules/generic_password.yar", "yara-rules/generic_username.yar"]
/// ```
pub fn rec_get_files_by_ext(dir: &str, ext: &str) -> Vec<String> {
    let mut discovered_files: Vec<String> = Vec::new();

    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let entry_path = entry.path();
        if let Some(file_ext) = entry_path.extension() {
            if file_ext == ext {
                if let Some(filepath) = entry_path.to_str() {
                    discovered_files.push(String::from(filepath));
                }
            }
        }
    }

    discovered_files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_returns_this_file_as_rust() {
        let actual: Vec<String> = rec_get_files_by_ext("src", "rs");
        assert!(actual.iter().any(|e| e == "src/utils.rs"));
    }

    #[test]
    fn it_does_not_return_this_file_as_txt() {
        let actual: Vec<String> = rec_get_files_by_ext("src", "txt");
        assert!(!actual.iter().any(|e| e == "src/utils.rs"));
    }
}