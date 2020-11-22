use std::fs;
use std::str;
use std::error;
use log::error;

use yara::{Compiler, Rules, Rule, YrString, YaraError};

use crate::utils;
use crate::errors;

struct Processor {
    engine: Rules
}

pub struct FlatMatch {
    /// `The yara::Rule` structure is complicated and largely unnecessary for our needs
    /// This struct is a flat(ter) representation of the above, that only stores the matched rule's
    /// name, tags and data (the actual matches)
    rule_name: String,
    tags: Vec<String>,
    data: Vec<String>
}

impl Processor {
    /// Constructs a Processor object whose rules have been loaded recursively
    /// from a directory
    ///
    /// # Arguments
    ///
    /// * `rule_root` - The root directory under which `.yar` files will be found
    ///
    /// # Examples
    ///
    /// ```
    /// let p: Processor = Processor::from_dir("yara-rules/");
    /// ```
    ///
    /// # Errors
    ///
    /// `crate::errors::NoYaraRulesError` - When no `.yar` files can be found under `rule_root`
    fn from_dir(rule_root: &str) -> Result<Processor, Box<dyn error::Error>> {
        let rule_files = utils::rec_get_files_by_ext(rule_root, "yar");

        if rule_files.is_empty() {
            error!("Found no .yar files under {}. Refusing to continue", rule_root);
            return Err(Box::new(errors::NoYaraRulesError));
        }

        Processor::with_rule_files(rule_files)
    }

    /// Constructs a Processor object whose rules have been loaded by
    /// the contents of the provided files
    /// Largely works the same as `Processor::from_dir`, but each file must
    /// be passed explicitly
    fn with_rule_files(filenames: Vec<String>) -> Result<Processor, Box<dyn error::Error>> {
        let mut rules: Vec<String> = Vec::new();
        for filename in filenames.into_iter() {
            rules.push(fs::read_to_string(filename)?);
        }

        Processor::with_rules(rules)
    }

    /// Constructs a Processor object from a string representing a Yara rule
    ///
    /// # Arguments
    ///
    /// * `rule` - The Yara rule
    #[allow(dead_code)]
    fn with_rule_str(rule: &str) -> Result<Processor, Box<dyn error::Error>> {
        Processor::with_rules(vec![rule.to_string()])
    }

    /// Constructs a Processor object from a vector of strings, each of which
    /// represents a Yara rule
    ///
    /// # Arguments
    ///
    /// * `rules` - A vector of Yara rule strings
    fn with_rules(rules: Vec<String>) -> Result<Processor, Box<dyn error::Error>> {
        let mut compiler = Compiler::new()?;

        for rule in rules.into_iter() {
            compiler.add_rules_str(&rule)?;
        }

        let engine = compiler.compile_rules()?;
        Ok(Processor { engine })
    }

    /// Given a string, tries to match the compiled Yara rules against it
    /// Returns the matches as a vector of `FlatMatch` objects
    ///
    /// # Arguments
    ///
    /// * `filestr` - The string against which the Yara matcher will run
    ///
    /// # Examples
    /// ```
    /// let p = Processor::with_rule_files("yara-rules/MyPassword.yar");
    /// let matches: Vec<FlatMatch> = p.process("password: HelloWorld").unwrap();
    /// for m in matches {
    ///     m.rule_name(); // "MyPassword"
    ///     m.tags(); // ["my", "matched", "rule", "tags"]
    ///     m.data(); // ["HelloWorld"]
    /// }
    /// ```
    fn process(&self, filestr: &str) -> Result<Vec<FlatMatch>, YaraError> {
        let rules: Vec<Rule> = self.engine.scan_mem(filestr.as_bytes(), 10)?;
        Ok(FlatMatch::from_rules(rules))
    }
}

impl FlatMatch {

    /// Used by `Processor#process` to convert `yara::Rule` objects
    /// to FlatMatch objects
    ///
    /// # Arguments
    ///
    /// * `rules` - A vector of the rules matched by the Yara engine
    fn from_rules(rules: Vec<Rule>) -> Vec<FlatMatch> {
        rules.into_iter().map(|r| FlatMatch::from_rule(r)).collect()
    }

    /// Consumes and converts a `yara::Rule` object into a `FlatMatch`
    ///
    /// # Arguments
    ///
    /// * `rule` - A yara::Rule object, as returned by `yara::Compiler.scan_mem`
    fn from_rule(rule: Rule) -> FlatMatch {
        let rule_name = format!("{}::{}", rule.namespace, rule.identifier);
        let tags: Vec<String> = rule.tags.iter().map(|&t| String::from(t)).collect();
        let mut byte_data: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();

        let rule_strings: Vec<YrString> = rule.strings;
        for rule_string in rule_strings.into_iter() {
            // We don't care about zero length matches
            if rule_string.matches.len() == 0 {
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
    pub fn tags(&self) -> &Vec<String> {
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
    fn new(rule_name: String, tags: Vec<String>, matches: &Vec<Vec<u8>>) -> FlatMatch {
        let mut data: Vec<String> = Vec::new();
        for single_match in matches.into_iter() {
            match str::from_utf8(&single_match) {
                Ok(match_string) => data.push(match_string.to_string()),
                Err(e) => error!("Could not convert byte array {:?} into string ({}) for Rule {}", single_match, e, rule_name)
            }
        }
        FlatMatch { rule_name, tags, data }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn password_rule() -> String {
        String::from(r#"
        rule MyPass
        {
            meta:
                name = "My Pass"

            strings:
                $a = /pw:.+/

            condition:
                $a
        }
        "#)
    }

    fn processor() -> Processor {
        Processor::with_rule_str(&password_rule()).unwrap()
    }

    #[test]
    fn processor_does_not_blow_up() {
        processor();
    }

    #[test]
    #[should_panic]
    fn processor_blows_up_with_bad_rule() {
        Processor::with_rule_str("Bad Rule").unwrap();
    }

    #[test]
    fn process_does_not_blow_up() {
        let p = processor();
        p.process(&"foo").unwrap();
    }

    #[test]
    fn process_returns_correct_data() {
        let p = processor();
        let matches = p.process(&"pw: helloworld").unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].rule_name(), String::from("default::MyPass"));
        assert_eq!(matches[0].tags().len(), 0);
        assert_eq!(*matches[0].data()[0], String::from("pw: helloworld"));
    }
}
