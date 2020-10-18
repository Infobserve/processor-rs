extern crate yaml_rust;
use std::error::Error;
use std::fmt;

use yaml_rust::YamlLoader;

const DEFAULT_NUM_PROCESSORS: usize = 2;
const DEFAULT_NUM_FEEDERS: usize = 4;

#[derive(PartialEq, Debug)]
pub struct Settings {
    num_processors: usize,
    num_feeders: usize
}

impl Settings {
    pub fn from_file(filename: &str) -> Result<Settings, Box<dyn Error>> {
        let contents  = std::fs::read_to_string(filename);
        
        // Return the default settings if the file can't be read
        if contents.is_err() {
            return Ok(Settings::default());
        }

        Settings::from_string(&contents.unwrap())
    }

    pub fn from_string(yml: &str) -> Result<Settings, Box<dyn Error>> {
        let docs = YamlLoader::load_from_str(&yml)?;

        // Return the default settings if the file is empty
        if docs.is_empty() {
            return Ok(Settings::default());
        }

        let workers = &docs[0]["workers"];
        let num_processors = match workers["processors"].as_i64() {
            Some(processors) => processors as i32,
            None => DEFAULT_NUM_PROCESSORS as i32
        };

        let num_feeders = match workers["feeders"].as_i64() {
            Some(feeders) => feeders as i32,
            None => DEFAULT_NUM_FEEDERS as i32
        };

        if num_processors <= 0 || num_feeders <= 0 {
            return Err(Box::new(NonPositiveWorkers));
        }

        Ok(Settings { num_processors: num_processors as usize, num_feeders: num_feeders as usize })
    }

    pub fn num_processors(&self) -> usize {
        self.num_processors
    }

    pub fn num_feeders(&self) -> usize {
        self.num_feeders
    }
    
    fn default() -> Settings {
        Settings {
            num_processors: DEFAULT_NUM_PROCESSORS,
            num_feeders: DEFAULT_NUM_FEEDERS
        }
    }
}

#[derive(Debug, Clone)]
struct NonPositiveWorkers;
impl fmt::Display for NonPositiveWorkers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Number of workers must be positive")
    }
}
impl Error for NonPositiveWorkers {}

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
        "#;
        assert_eq!(Settings::from_string(yml).unwrap(), Settings { num_processors: 2, num_feeders: DEFAULT_NUM_FEEDERS });
    }

    #[test]
    fn returns_correct_feeder_value() {
        let yml = r#"
        workers:
            feeders: 5
        "#;
        assert_eq!(Settings::from_string(yml).unwrap(), Settings { num_processors: DEFAULT_NUM_PROCESSORS, num_feeders: 5 })
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