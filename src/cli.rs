extern crate clap;

use clap::{crate_authors, App, Arg};

pub struct Cli {
    pub config_path: String,
}

impl Cli {
    pub fn new() -> Cli {
        let a = App::new("Infobserve Processor")
            .version("1.0")
            .author(crate_authors!())
            .about("Invokes the Infobserve processor process")
            .arg(
                Arg::new("config")
                    .short('c')
                    .long("config")
                    .value_name("CONFIG")
                    .about("Sets a custom config file")
                    .default_value("config.yaml"),
            )
            .get_matches();

        Cli {
            config_path: a
                .value_of("config")
                .expect("configuration path not valid value!")
                .to_string(),
        }
    }
}
