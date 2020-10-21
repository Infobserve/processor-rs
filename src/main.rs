mod settings;
mod event;
mod errors;
mod utils;

use settings::Settings;

fn main() {
    let _s = Settings::from_file("config.yaml").unwrap();

    let v: Vec<String> = utils::rec_get_files_by_ext("src", "rs");

    for f in v.into_iter() {
        println!("{}", f);
    }
}
