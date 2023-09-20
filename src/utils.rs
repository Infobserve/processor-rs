//! Contains varius utility/helper functions

use std::cmp;

use walkdir::WalkDir;

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

/// Clamps the given value over the given minimum value
/// Returns the given value if it is over `min`, otherwise returns `min`
/// 
/// # Example
/// ```
/// use utils::clamp_min;
/// 
/// assert_eq!(2, clamp_min(2, 0));
/// assert_eq!(0, clamp_min(-2, 0));
/// ```
pub fn clamp_min<T: cmp::Ord>(val: T, min: T) -> T {
    if val < min {
        min
    } else {
        val
    }
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
    fn clamps_when_below_min() {
        assert_eq!(2, clamp_min(2, 0));
    }

    #[test]
    fn does_not_clamp_when_below_min() {
        assert_eq!(0, clamp_min(-2, 0));
    }
}