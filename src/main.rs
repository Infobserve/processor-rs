mod settings;
mod event;
mod errors;
mod utils;
mod processing;

use processing::{Processor, FlatMatch};
use log::error;

fn main() {
    let s = settings::Settings::from_file("config.yaml").unwrap_or_else(|e| {
        error!("Could not load configuration file: {}", e);
        std::process::exit(1);
    });


    println!("{}", s.yara_rule_dir());

    match Processor::from_dir(s.yara_rule_dir()) {
        Ok(p) => {
            let results: Vec<FlatMatch> = p.process("password: hello").unwrap();

            for result in results.iter() {
                println!("Tags: {:?}", result.tags());
                println!("Rule name: {:?}", result.rule_name());
                println!("Data: {:?}", result.data());
            }
        },
        Err(e) => println!("Err: {}", e)
    }
}
