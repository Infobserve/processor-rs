mod settings;
mod event;
mod errors;
mod utils;
mod processing;

use log::error;

fn main() {
    let s = settings::Settings::from_file("config.yaml").unwrap_or_else(|e| {
        error!("Could not load configuration file: {}", e);
        std::process::exit(1);
    });


    let (sender, receiver) = crossbeam_channel::unbounded();
    let p_handles = processing::start_processors(&receiver, s.yara_rule_dir(), s.num_processors());

    // Dropping the sender will gracefully close the receiver's end as well
    // and as such make all processor threads return
    drop(sender);

    // Wait for all threads to return
    for handle in p_handles {
        handle.join().unwrap();
        println!("Joined processor");
    }
}
