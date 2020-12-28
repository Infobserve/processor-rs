use log::{info, warn};
use std::fs;

use anyhow::Result;

use yaml_rust::YamlLoader;
use crate::errors;

const DEFAULT_NUM_PROCESSORS: i32 = 1;
const DEFAULT_NUM_FEEDERS: i32 = 1;
const DEFAULT_YARA_RULE_DIR: &str = "yara-rules/";

#[derive(PartialEq, Debug)]
pub struct Settings {
    yara_rule_dir: String,
    num_processors: i32,
    num_feeders: i32
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            yara_rule_dir: String::from(DEFAULT_YARA_RULE_DIR),
            num_processors: DEFAULT_NUM_PROCESSORS,
            num_feeders: DEFAULT_NUM_FEEDERS
        }
    }
}

impl Settings {
    /// Loads configuration from a YAML file.
    /// If the file cannot be read, the default settings are returned instead
    ///
    /// # Arguments
    ///
    /// * `filename`: The fully qualified path to the file to read from
    ///
    /// # Returns
    /// anyhow::Result<Settings>: Will only be Err if the number of any worker (feeder, processor
    /// or loader) is negative
    pub fn from_file(filename: &str) -> Result<Self> {
        match fs::read_to_string(filename) {
            Ok(contents) => Settings::from_string(&contents),
            Err(e) => {
                info!("Could not read configuration file {} ({}). Loading defaults", filename, e);
                Ok(Default::default())
            }
        }
    }

    #[allow(dead_code)]
    pub fn num_processors(&self) -> i32 {
        self.num_processors
    }

    #[allow(dead_code)]
    pub fn num_feeders(&self) -> i32 {
        self.num_feeders
    }

    #[allow(dead_code)]
    pub fn yara_rule_dir(&self) -> &str {
        &self.yara_rule_dir
    }

    fn from_string(yml: &str) -> Result<Settings> {
        let docs = YamlLoader::load_from_str(&yml)?;

        // Return the default settings if the file is empty
        if docs.is_empty() {
            warn!("Found empty configuration file. Loading default settings");
            return Ok(Default::default());
        }

        let doc = &docs[0];

        let rule_dir = doc["yara_rule_dir"].as_str();
        let processors = doc["workers"]["processors"].as_i64();
        let feeders = doc["workers"]["feeders"].as_i64();

        Settings::build_from_config(rule_dir, processors, feeders)
    }

    fn build_from_config(
        rule_dir: Option<&str>,
        processors: Option<i64>,
        feeders: Option<i64>
    ) -> Result<Settings> {
        let yara_rule_dir = rule_dir.unwrap_or(DEFAULT_YARA_RULE_DIR);
        let num_processors = processors.unwrap_or(DEFAULT_NUM_PROCESSORS as i64) as i32;
        let num_feeders = feeders.unwrap_or(DEFAULT_NUM_FEEDERS as i64) as i32;

        if num_processors <= 0 || num_feeders <= 0 {
            return Err(errors::NonPositiveWorkersError.into());
        }

        Ok(Settings {
            yara_rule_dir: String::from(yara_rule_dir),
            num_processors: num_processors,
            num_feeders: num_feeders
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_returns_the_default_for_missing_file() {
        assert_eq!(Settings::from_file("non-existent.yml").unwrap(), Default::default());
    }

    #[test]
    fn it_returns_the_default_for_empty_cfg() {
        let yml = "";
        assert_eq!(Settings::from_string(yml).unwrap(), Default::default());
    }

    #[test]
    fn returns_correct_processor_value() {
        let yml = r#"
        workers:
            processors: 2
        yara_rule_dir: foo
        "#;
        assert_eq!(
            Settings::from_string(yml).unwrap(),
            Settings {
                yara_rule_dir: String::from("foo"),
                num_processors: 2,
                num_feeders: DEFAULT_NUM_FEEDERS
            }
        );
    }

    #[test]
    fn returns_correct_feeder_value() {
        let yml = r#"
        workers:
            feeders: 5
        "#;
        assert_eq!(
            Settings::from_string(yml).unwrap(),
            Settings {
                yara_rule_dir: String::from(DEFAULT_YARA_RULE_DIR),
                num_processors: DEFAULT_NUM_PROCESSORS,
                num_feeders: 5
            }
        )
    }

    #[test]
    #[should_panic]
    fn blows_up_for_non_positive_workers() {
        let  yml = r#"
        workers:
            feeders: -1
        "#;
        Settings::from_string(yml).unwrap();
    }
}
