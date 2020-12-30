//! Contains varius utility/helper functions
use walkdir::WalkDir;
use inflector::string::pluralize::to_plural;

/// Recursively finds and returns the relative path
/// to all files that satisfy the `ext` extension filter
///
/// # Arguments
///
/// * `dir` - The path to the root directory under which, files will be searched
/// * `ext` - The extension to look for (no leading '.')
///
/// # Examples
///
/// ```
/// use utils::rec_get_files_by_ext;
/// 
/// let rule_files: Vec<&str> = rec_get_files_by_ext("yara-rules", "yar");
/// assert_eq!(rule_files, vec!["yara-rules/generic_password.yar"])
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

/// Uses Inflector's `to_plural` to pluralize `word` based on `num`
///
/// Note: [Inflector's repository](https://github.com/whatisinternet/Inflector) seems abandoned unfortunately,
///         but it seems to work well enough for our current needs.
///         That being said, there's an 1-year-old open [PR](https://github.com/whatisinternet/Inflector/pull/73)
///         that fixes some issues with pluralization (e.g. mouse -> mouseice).
///         If the pluralization is wrong, it's *probably* not a bug with this function but the
///         `Inflector` crate itself (read the note above)
///
/// # Arguments
///
/// * `num` - Any number type that can be cast into i64. If it's `1`,
///           the word will not be pluralized
/// * `word` - The word to pluralize. **Do not** use already pluralized words as this method will
///            not singularize them
///
/// # Examples
/// use crate::utils::pluralize;
///
/// ```
/// assert_eq!(pluralize(5, "cactus"), "5 cacti")
/// assert_eq!(pluralize(1, "loader"), "1 loader")
/// // However
/// assert_eq!(pluralize(1, "loaders"), "1 loaders")
/// ```
pub fn pluralize<T: Into<i64>>(num: T, word: &str) -> String {
    let x = num.into() as i64;
    let w = match x {
        1 => word.to_owned(),
        _ => to_plural(word)
    };

    format!("{} {}", x, w)
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

    #[test]
    fn it_doesnt_pluralize_on_1() {
        let actual = pluralize(1, "word");
        assert_eq!(actual, "1 word")
    }

    #[test]
    fn it_pluralizes_on_5() {
        let actual = pluralize(5, "word");
        assert_eq!(actual, "5 words");
    }

    #[test]
    fn it_pluralizes_on_0() {
        let actual = pluralize(0, "word");
        assert_eq!(actual, "0 words");
    }
}