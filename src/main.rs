mod settings;
mod event;
mod errors;

use settings::Settings;

fn main() {
    let s = Settings::from_file("config.yaml").unwrap();

    println!("Yara rule dir: {}", s.yara_rule_dir());
}
