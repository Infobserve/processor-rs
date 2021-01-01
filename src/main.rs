use log::error;

mod settings;
mod errors;
mod utils;
mod processing;
mod database;
mod entities;
mod logger;

use database::{DbLoader, DbConnection};

fn main() {
    logger::init().unwrap();
    let s = settings::Settings::from_file("config.yaml").unwrap_or_else(|e| {
        error!("Could not load configuration file: {}", e);
        std::process::exit(1);
    });

    let connection = DbConnection::connect(s.db_user(), s.db_passwd(), s.db_database(), s.db_host(), s.db_port()).unwrap();
    let db_loader = DbLoader::with_connection(connection);

    if let Err(e) = db_loader.create_schema() {
        error!("Could not create schema: {}", e);
        std::process::exit(1);
    }

    let (feed_sendr, feed_recvr) = crossbeam_channel::unbounded();
    let (load_sendr, load_recvr) = crossbeam_channel::unbounded();

    let p_handles = processing::start_processors(
        &feed_recvr,
        &load_sendr,
        s.yara_rule_dir(),
        s.num_processors()
    );

    let l_handles = database::start_loaders(&load_recvr, db_loader, s.num_loaders());

    // Dropping the sender will gracefully close the receiver's end as well
    // and as such make all processor threads return
    drop(feed_sendr);

    // It is important to wait for all processor threads to join cleanly before
    // dropping the loader sender. If we drop both senders together, processor threads
    // that have events left in their queue will panic when they try to send matching ones
    // to the loader through the load channel
    for handle in p_handles {
        handle.join().unwrap();
        println!("Joined processor");
    }

    drop(load_sendr);

    for handle in l_handles {
        handle.join().unwrap();
        println!("Joined loader");
    }
}
