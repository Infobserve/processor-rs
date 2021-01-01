use log::{info, warn};
use std::fs;
use std::env;

use anyhow::Result;
use yaml_rust::{YamlLoader, Yaml};

use crate::errors;

const DEFAULT_NUM_PROCESSORS: i32 = 1;
const DEFAULT_NUM_FEEDERS: i32 = 1;
const DEFAULT_NUM_LOADERS: i32 = 1;
const DEFAULT_YARA_RULE_DIR: &str = "yara-rules/";

const DEFAULT_DB_USER: &str = "postgres";
const DEFAULT_DB_PASSWD: &str = "infobserve";
const DEFAULT_DB_DATABASE: &str = "postgres";
const DEFAULT_DB_HOST: &str = "localhost";
const DEFAULT_DB_PORT: u16 = 5432;

#[derive(PartialEq, Debug)]
pub struct Settings {
    yara_rule_dir: String,
    worker_settings: WorkerSettings,
    db_settings: DbSettings
}

#[derive(PartialEq, Debug)]
struct DbSettings {
    user: String,
    passwd: String,
    database: String,
    host: String,
    port: u16
}

#[derive(PartialEq, Debug)]
struct WorkerSettings {
    num_processors: i32,
    num_feeders: i32,
    num_loaders: i32
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

    pub fn num_processors(&self) -> i32 {
        self.worker_settings.num_processors
    }

    #[allow(dead_code)]
    pub fn num_feeders(&self) -> i32 {
        self.worker_settings.num_feeders
    }

    pub fn num_loaders(&self) -> i32 {
        self.worker_settings.num_loaders
    }

    pub fn yara_rule_dir(&self) -> &str {
        &self.yara_rule_dir
    }

    pub fn db_user(&self) -> &str {
        &self.db_settings.user
    }

    pub fn db_passwd(&self) -> &str {
        &self.db_settings.passwd
    }

    pub fn db_database(&self) -> &str {
        &self.db_settings.database
    }

    pub fn db_host(&self) -> &str {
        &self.db_settings.host
    }

    pub fn db_port(&self) -> u16 {
        self.db_settings.port
    }

    fn from_string(yml: &str) -> Result<Settings> {
        let docs = YamlLoader::load_from_str(&yml)?;

        // Return the default settings if the file is empty
        if docs.is_empty() {
            warn!("Found empty configuration file. Loading default settings");
            return Ok(Default::default());
        }

        let doc = &docs[0];

        let rule_dir = doc["yara_rule_dir"].as_str().unwrap_or(DEFAULT_YARA_RULE_DIR);
        let worker_settings = WorkerSettings::from_block(&doc["workers"])?;
        let db_settings = DbSettings::from_block(&doc["database"]);

        Ok(Self {
            yara_rule_dir: rule_dir.to_owned(),
            worker_settings,
            db_settings
        })
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            yara_rule_dir: DEFAULT_YARA_RULE_DIR.to_owned(),
            db_settings: Default::default(),
            worker_settings: Default::default()
        }
    }
}

impl WorkerSettings {
    fn from_block(block: &Yaml) -> Result<Self> {
        let num_processors = Self::int_or_default(&block["processors"], DEFAULT_NUM_PROCESSORS);
        let num_feeders = Self::int_or_default(&block["feeders"], DEFAULT_NUM_FEEDERS);
        let num_loaders = Self::int_or_default(&block["loaders"], DEFAULT_NUM_LOADERS);

        if num_processors <= 0 || num_feeders <= 0 || num_loaders <= 0 {
            return Err(errors::NonPositiveWorkersError.into());
        }

        Ok(Self {
            num_processors,
            num_feeders,
            num_loaders
        })
    }

    fn int_or_default(block: &Yaml, default: i32) -> i32 {
        block.as_i64().unwrap_or(default as i64) as i32
    }
}

impl Default for WorkerSettings {
    fn default() -> Self {
        Self {
            num_processors: DEFAULT_NUM_PROCESSORS,
            num_feeders: DEFAULT_NUM_FEEDERS,
            num_loaders: DEFAULT_NUM_LOADERS
        }
    }
}


impl DbSettings {
    fn from_block(yaml_block: &Yaml) -> Self {
        let user = match yaml_block["user"].as_str() {
            Some(u) => u,
            None => DEFAULT_DB_USER
        }.to_owned();
        let passwd = match yaml_block["passwd"].as_str() {
            Some(p) => p.to_owned(),
            None => {
                match env::var("INFOBSERVE_POSTGRES_PASSWD") {
                    Ok(v) => v,
                    Err(_) => DEFAULT_DB_PASSWD.to_owned()
                }
            }
        };
        let database = match yaml_block["database"].as_str() {
            Some(d) => d,
            None => DEFAULT_DB_DATABASE
        }.to_owned();
        let host = match yaml_block["host"].as_str() {
            Some(h) => h,
            None => DEFAULT_DB_HOST
        }.to_owned();
        let port = match yaml_block["port"].as_i64() {
            Some(p) => p as u16,
            None => DEFAULT_DB_PORT
        };

        Self {
            user,
            passwd,
            database,
            host,
            port
        }
    }
}

impl Default for DbSettings {
    fn default() -> Self {
        Self {
            user: DEFAULT_DB_USER.to_owned(),
            passwd: DEFAULT_DB_PASSWD.to_owned(),
            database: DEFAULT_DB_DATABASE.to_owned(),
            host: DEFAULT_DB_HOST.to_owned(),
            port: DEFAULT_DB_PORT
        }
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
            loaders: 5
        yara_rule_dir: foo
        "#;

        let worker_settings = WorkerSettings {
            num_processors: 2,
            num_feeders: DEFAULT_NUM_FEEDERS,
            num_loaders: 5
        };

        assert_eq!(
            Settings::from_string(yml).unwrap(),
            Settings {
                yara_rule_dir: String::from("foo"),
                worker_settings,
                db_settings: Default::default()
            }
        );
    }

    #[test]
    fn returns_correct_feeder_value() {
        let yml = r#"
        workers:
            feeders: 5
        "#;
        let worker_settings = WorkerSettings {
            num_processors: DEFAULT_NUM_PROCESSORS,
            num_feeders: 5,
            num_loaders: DEFAULT_NUM_LOADERS
        };
        assert_eq!(
            Settings::from_string(yml).unwrap(),
            Settings {
                yara_rule_dir: String::from(DEFAULT_YARA_RULE_DIR),
                worker_settings,
                db_settings: Default::default()
            }
        )
    }

    #[test]
    fn returns_correct_db_values() {
        let yml = r#"
        database:
            host: localhost
            port: 1337
            database: my_db
            user: my_user
            passwd: my_passwd
        "#;

        let db_settings = DbSettings {
            user: "my_user".to_owned(),
            passwd: "my_passwd".to_owned(),
            database: "my_db".to_owned(),
            host: "localhost".to_owned(),
            port: 1337
        };

        assert_eq!(
            Settings::from_string(yml).unwrap(),
            Settings {
                yara_rule_dir: String::from(DEFAULT_YARA_RULE_DIR),
                db_settings,
                worker_settings: Default::default()
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
