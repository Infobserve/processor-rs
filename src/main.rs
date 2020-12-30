mod settings;
mod errors;
mod utils;
mod processing;
mod database;
mod logger;

use database::DbConnection;

fn main() {
    logger::init().unwrap();
    let s = settings::Settings::from_file("config.yaml").unwrap_or_else(|e| {
        error!("Could not load configuration file: {}", e);
        std::process::exit(1);
    });

    let connection = DbConnection::connect(s.db_user(), s.db_passwd(), s.db_database(), s.db_host(), s.db_port()).unwrap();
    let (feed_sendr, feed_recvr) = crossbeam_channel::unbounded();

    let p_handles = processing::start_processors(
        &feed_recvr,
        s.yara_rule_dir(),
        s.num_processors()
    );

    // Dropping the sender will gracefully close the receiver's end as well
    // and as such make all processor threads return
    drop(feed_sendr);

    // Wait for all threads to return
    for handle in p_handles {
        handle.join().unwrap();
        println!("Joined processor");
    }
}
