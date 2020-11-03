use std::error;
use std::fs;

use yaml_rust::YamlLoader;
use crate::errors;

const DEFAULT_NUM_PROCESSORS: usize = 1;
const DEFAULT_NUM_FEEDERS: usize = 1;
const DEFAULT_YARA_RULE_DIR: &str = "yara-rules/";

#[derive(PartialEq, Debug)]
pub struct Settings {
    yara_rule_dir: String,
    num_processors: usize,
    num_feeders: usize
}

impl Settings {
    /// Loads configuration from a file.
    /// If the file cannot be read (it doesn't exist or the current user cannot read it),
    /// the default settings are instead returned
    ///
    /// # Arguments
    ///
    /// * `filename`: The fully qualified path to the file to read from
    pub fn from_file(filename: &str) -> Result<Settings, Box<dyn error::Error>> {
        match fs::read_to_string(filename) {
            Ok(contents) => Settings::from_string(&contents),
            Err(_) => {
                log::info!("Could not read configuration file ({}). Loading defaults", filename);
                Ok(Settings::default())
            }
        }
    }

    #[allow(dead_code)]
    pub fn num_processors(&self) -> usize {
        self.num_processors
    }

    #[allow(dead_code)]
    pub fn num_feeders(&self) -> usize {
        self.num_feeders
    }

    #[allow(dead_code)]
    pub fn yara_rule_dir(&self) -> &str {
        &self.yara_rule_dir
    }

    fn from_string(yml: &str) -> Result<Settings, Box<dyn error::Error>> {
        let docs = YamlLoader::load_from_str(&yml)?;

        // Return the default settings if the file is empty
        if docs.is_empty() {
            return Ok(Settings::default());
        }

        let doc = &docs[0];

        let rule_dir = doc["yara_rule_dir"].as_str();
        let processors = doc["workers"]["processors"].as_i64();
        let feeders = doc["workers"]["feeders"].as_i64();

        Settings::build_from_config(rule_dir, processors, feeders)
    }

    fn build_from_config(rule_dir: Option<&str>, processors: Option<i64>,
                         feeders: Option<i64>) -> Result<Settings, Box<dyn error::Error>> {
        let yara_rule_dir = match rule_dir {
            Some(y) => y,
            None => DEFAULT_YARA_RULE_DIR
        };
        let num_processors = match processors {
            Some(p) => p as i32,
            None => DEFAULT_NUM_PROCESSORS as i32
        };
        let num_feeders = match feeders {
            Some(p) => p as i32,
            None => DEFAULT_NUM_FEEDERS as i32
        };

        if num_processors <= 0 || num_feeders <= 0 {
            return Err(Box::new(errors::NonPositiveWorkersError));
        }

        Ok(Settings {
            yara_rule_dir: String::from(yara_rule_dir),
            num_processors: num_processors as usize,
            num_feeders: num_feeders as usize 
        })
    }
    
    fn default() -> Settings {
        Settings {
            yara_rule_dir: String::from(DEFAULT_YARA_RULE_DIR),
            num_processors: DEFAULT_NUM_PROCESSORS,
            num_feeders: DEFAULT_NUM_FEEDERS
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_returns_the_default_for_missing_file() {
        assert_eq!(Settings::from_file("non-existent.yml").unwrap(), Settings::default());
    }

    #[test]
    fn it_returns_the_default_for_empty_cfg() {
        let yml = "";
        assert_eq!(Settings::from_string(yml).unwrap(), Settings::default());
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
            Settings { yara_rule_dir: String::from("foo"), num_processors: 2, num_feeders: DEFAULT_NUM_FEEDERS }
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
            Settings { yara_rule_dir: String::from(DEFAULT_YARA_RULE_DIR), num_processors: DEFAULT_NUM_PROCESSORS, num_feeders: 5 }
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
