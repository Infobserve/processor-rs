extern crate clap;

use clap::{crate_authors, App, Arg};

pub struct Cli {
    config_path: String,
}

impl Cli {
    pub fn config_path(&self) -> &str {
        &self.config_path
    }
}

impl Cli {
    pub fn parse_args() -> Cli {
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
            // We unwrap because it is handled by the clap package.
            // The case of giving a -c without value uses the default value.
            config_path: a
                .value_of("config")
                .unwrap()
                .to_string(),
        }
    }
}
