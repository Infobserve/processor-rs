extern crate yaml_rust;
use std::error::Error;

use yaml_rust::YamlLoader;

pub trait Parseable {
    fn from_yaml(filename: &'static str) -> Result<Self, Box<dyn Error>> where Self: Sized;
    fn default() -> Self;
}

const DEFAULT_NUM_PROCESSORS: usize = 2;
const DEFAULT_NUM_FEEDERS: usize = 4;

#[derive(Debug)]
pub struct Settings {
    pub num_processors: usize,
    pub num_feeders: usize
}

impl Parseable for Settings {
    fn default() -> Settings {
        Settings {
            num_processors: DEFAULT_NUM_PROCESSORS,
            num_feeders: DEFAULT_NUM_FEEDERS
        }
    }

    fn from_yaml(filename: &str) -> Result<Settings, Box<dyn Error>> {
        let contents = std::fs::read_to_string(filename);
        
        // Return the default settings if the file can't be read
        if contents.is_err() {
            return Ok(Settings::default());
        }

        let docs = YamlLoader::load_from_str(&contents.unwrap())?;

        // Same thing if the file has no content
        if docs.len() == 0 {
            return Ok(Settings::default());
        }

        let doc = &docs[0];
        let workers = &doc["workers"];
        let num_processors = match workers["processors"].as_i64() {
            Some(processors) => processors as usize,
            None => DEFAULT_NUM_PROCESSORS
        };

        let num_feeders = match workers["feeders"].as_i64() {
            Some(feeders) => feeders as usize,
            None => DEFAULT_NUM_FEEDERS
        };

        Ok(Settings { num_processors, num_feeders })
    }
}