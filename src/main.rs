use log::error;

mod cli;
mod config;
mod errors;
mod utils;
mod processing;
mod database;
mod entities;
mod logger;

use std::process;

use cli::Cli;
use config::Config;
use database::{DbLoader, DbConnection};

fn main() {
    let cli: Cli = Cli::new();

    if let Err(e) = logger::init() {
        error!("Could not initialize logger: {}", e);
        process::exit(1);
    }

    let cfg = match Config::from_file(&cli.config_path) {
        Ok(c) => c,
        Err(e) => {
            error!("Could not load configuration file: {}", e);
            process::exit(1);
        }
    };

    let connection = match DbConnection::connect(
        cfg.db().user(),
        cfg.db().passwd(),
        cfg.db().db_name(),
        cfg.db().host(),
        cfg.db().port()) {
        Ok(c) => c,
        Err(e) => {
            error!("Could not connect to database: {}", e);
            process::exit(1);
        }
    };

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
        cfg.yara_rule_dir(),
        cfg.workers().num_processors() as usize
    );

    let l_handles = database::start_loaders(&load_recvr, db_loader, cfg.workers().num_loaders());

    // Dropping the sender will gracefully close the receiver's end as well
    // and as such make all processor threads return
    drop(feed_sendr);

    // It is important to wait for all processor threads to join cleanly before
    // dropping the loader sender. If we drop both senders together, processor threads
    // that have events left in their queue will panic when they try to send matching ones
    // to the loader through the load channel
    for handle in p_handles {
        if let Ok(res) = handle.join() {
            match res {
                Ok(s) => println!("{}", s),
                Err(e) => println!("Error in processor: {}", e)
            }
        }
        println!("Joined processor");
    }

    drop(load_sendr);

    for handle in l_handles {
        handle.join().unwrap();
        println!("Joined loader");
    }
}
