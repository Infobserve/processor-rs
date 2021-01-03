use std::str;
use yara::{Rule, YrString};
use log::error;

/// `The yara::Rule` structure is complicated and largely unnecessary for our needs
/// This struct is a flat(ter) representation of the above, that only stores the matched rule's
/// name, tags and data (the actual matches)
#[derive(Debug)]
pub struct FlatMatch {
    rule_name: String,
    tags: Vec<String>,
    data: Vec<String>
}

impl FlatMatch {

    /// Used by `Processor#process` to convert `yara::Rule` objects
    /// to FlatMatch objects
    ///
    /// # Arguments
    ///
    /// * `rules` - A vector of the rules matched by the Yara engine
    pub fn from_rules(rules: Vec<Rule>) -> Vec<FlatMatch> {
        rules.into_iter().map(FlatMatch::from_rule).collect()
    }

    /// Consumes and converts a `yara::Rule` object into a `FlatMatch`
    ///
    /// # Arguments
    ///
    /// * `rule` - A yara::Rule object, as returned by `yara::Compiler.scan_mem`
    pub fn from_rule(rule: Rule) -> FlatMatch {
        let rule_name = format!("{}::{}", rule.namespace, rule.identifier);
        let tags: Vec<String> = rule.tags.iter().map(|&t| String::from(t)).collect();
        let mut byte_data: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();

        let rule_strings: Vec<YrString> = rule.strings;
        for rule_string in rule_strings.into_iter() {
            // We don't care about zero length matches
            if rule_string.matches.is_empty() {
                continue;
            }
            let rule_matches = rule_string.matches;

            for single_match in rule_matches.into_iter() {
                byte_data.push(single_match.data);
            }
        }

        FlatMatch::new(rule_name, tags, &byte_data)
    }

    #[allow(dead_code)]
    pub fn rule_name(&self) -> &str {
        &self.rule_name
    }

    #[allow(dead_code)]
    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    #[allow(dead_code)]
    pub fn data(&self) -> &Vec<String> {
        &self.data
    }

    /// Constructs a new `FlatMatch` object by iterating over the first dimension of `matches`,
    /// and converting each element of the second from a byte array to a string
    ///
    /// If a byte array does not represent a valid unicode byte sequence, it it dropped
    /// and a warning is emitted. We should be expecting a few of those until we support binary
    /// events as well
    ///
    /// # Arguments
    ///
    /// * `rule_name` - The name of the Yara Rule matched
    /// * `tags` - The tags within the rule that matched
    /// * `matches` - A vector of u8 vectors (2D), each of which represents a unicode string
    ///               | \x48 | \x61 | \x78 | \x30 | 0x72 |
    ///               | \x31 | \x33 | \x33 | \x37 |
    ///               | ... |
    ///
    /// # Examples
    /// ```
    /// let fm = FlatMatcH::new(String::from("MyRule"), vec!["hey", "ya"], vec![vec![66, 6f, 6f], vec![62, 61, 72]])
    /// assert_eq!(fm.data, ["foo".to_string(), "bar".to_string()])
    /// ```
    fn new(rule_name: String, tags: Vec<String>, matches: &[Vec<u8>]) -> FlatMatch {
        let mut data: Vec<String> = Vec::new();
        for single_match in matches.iter() {
            match str::from_utf8(&single_match) {
                Ok(match_string) => data.push(match_string.to_string()),
                Err(e) => error!("Could not convert byte array {:?} into string ({}) for Rule {}", single_match, e, rule_name)
            }
        }
        FlatMatch { rule_name, tags, data }
    }
}
