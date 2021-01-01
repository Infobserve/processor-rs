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
pub struct Config {
    yara_rule_dir: String,
    worker_cfg: WorkerCfg,
    db_cfg: DbCfg
}

#[derive(PartialEq, Debug)]
pub struct DbCfg {
    user: String,
    passwd: String,
    db_name: String,
    host: String,
    port: u16
}

#[derive(PartialEq, Debug)]
pub struct WorkerCfg {
    num_processors: i32,
    num_feeders: i32,
    num_loaders: i32
}

impl Config {
    /// Loads configuration from a YAML file.
    /// If the file cannot be read, the default settings are returned instead
    ///
    /// # Arguments
    ///
    /// * `filename`: The fully qualified path to the file to read from
    ///
    /// # Returns
    /// anyhow::Result<Config>: Will only be Err if the number of any worker (feeder, processor
    /// or loader) is negative
    pub fn from_file(filename: &str) -> Result<Self> {
        match fs::read_to_string(filename) {
            Ok(contents) => Config::from_string(&contents),
            Err(e) => {
                info!("Could not read configuration file {} ({}). Loading defaults", filename, e);
                Ok(Default::default())
            }
        }
    }

    pub fn workers(&self) -> &WorkerCfg {
        &self.worker_cfg
    }

    pub fn db(&self) -> &DbCfg {
        &self.db_cfg
    }

    pub fn yara_rule_dir(&self) -> &str {
        &self.yara_rule_dir
    }

    fn from_string(yml: &str) -> Result<Self> {
        let docs = YamlLoader::load_from_str(&yml)?;

        // Return the default settings if the file is empty
        if docs.is_empty() {
            warn!("Found empty configuration file. Loading default configuration");
            return Ok(Default::default());
        }

        let doc = &docs[0];

        let rule_dir = doc["yara_rule_dir"].as_str().unwrap_or(DEFAULT_YARA_RULE_DIR);
        let worker_cfg = WorkerCfg::from_block(&doc["workers"])?;
        let db_cfg = DbCfg::from_block(&doc["database"]);

        Ok(Self {
            yara_rule_dir: rule_dir.to_owned(),
            worker_cfg,
            db_cfg
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            yara_rule_dir: DEFAULT_YARA_RULE_DIR.to_owned(),
            db_cfg: Default::default(),
            worker_cfg: Default::default()
        }
    }
}

impl WorkerCfg {
    pub fn num_processors(&self) -> i32 {
        self.num_processors
    }

    #[allow(dead_code)]
    pub fn num_feeders(&self) -> i32 {
        self.num_feeders
    }

    pub fn num_loaders(&self) -> i32 {
        self.num_loaders
    }

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

impl Default for WorkerCfg {
    fn default() -> Self {
        Self {
            num_processors: DEFAULT_NUM_PROCESSORS,
            num_feeders: DEFAULT_NUM_FEEDERS,
            num_loaders: DEFAULT_NUM_LOADERS
        }
    }
}


impl DbCfg {
    pub fn user(&self) -> &str {
        &self.user
    }

    pub fn passwd(&self) -> &str {
        &self.passwd
    }

    pub fn db_name(&self) -> &str {
        &self.db_name
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

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
        let db_name = match yaml_block["db_name"].as_str() {
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
            db_name,
            host,
            port
        }
    }
}

impl Default for DbCfg {
    fn default() -> Self {
        Self {
            user: DEFAULT_DB_USER.to_owned(),
            passwd: DEFAULT_DB_PASSWD.to_owned(),
            db_name: DEFAULT_DB_DATABASE.to_owned(),
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
        assert_eq!(Config::from_file("non-existent.yml").unwrap(), Default::default());
    }

    #[test]
    fn it_returns_the_default_for_empty_cfg() {
        let yml = "";
        assert_eq!(Config::from_string(yml).unwrap(), Default::default());
    }

    #[test]
    fn returns_correct_processor_value() {
        let yml = r#"
        workers:
            processors: 2
            loaders: 5
        yara_rule_dir: foo
        "#;

        let worker_cfg = WorkerCfg {
            num_processors: 2,
            num_feeders: DEFAULT_NUM_FEEDERS,
            num_loaders: 5
        };

        assert_eq!(
            Config::from_string(yml).unwrap(),
            Config {
                yara_rule_dir: String::from("foo"),
                worker_cfg,
                db_cfg: Default::default()
            }
        );
    }

    #[test]
    fn returns_correct_feeder_value() {
        let yml = r#"
        workers:
            feeders: 5
        "#;
        let worker_cfg = WorkerCfg {
            num_processors: DEFAULT_NUM_PROCESSORS,
            num_feeders: 5,
            num_loaders: DEFAULT_NUM_LOADERS
        };

        assert_eq!(
            Config::from_string(yml).unwrap(),
            Config {
                yara_rule_dir: String::from(DEFAULT_YARA_RULE_DIR),
                worker_cfg,
                db_cfg: Default::default()
            }
        )
    }

    #[test]
    fn returns_correct_db_values() {
        let yml = r#"
        database:
            host: localhost
            port: 1337
            db_name: my_db
            user: my_user
            passwd: my_passwd
        "#;

        let db_cfg = DbCfg {
            user: "my_user".to_owned(),
            passwd: "my_passwd".to_owned(),
            db_name: "my_db".to_owned(),
            host: "localhost".to_owned(),
            port: 1337
        };

        assert_eq!(
            Config::from_string(yml).unwrap(),
            Config {
                yara_rule_dir: String::from(DEFAULT_YARA_RULE_DIR),
                db_cfg,
                worker_cfg: Default::default()
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
        Config::from_string(yml).unwrap();
    }
}
