mod settings;
mod event;
use settings::Settings;

fn main() {
    let s = Settings::from_file("config.yaml").unwrap();

    println!("Read settings:\nNumber of processors: {} & Number of feeders: {}", s.num_processors(), s.num_feeders());
}
