use log::error;

mod cli;
mod config;
mod errors;
mod utils;
mod processing;
mod database;
mod entities;
mod logger;
mod feeder;

use std::process;

use cli::Cli;
use config::Config;
use database::{DbLoader, DbConnection};

fn main() {
    let cli: Cli = Cli::parse_args();

    if let Err(e) = logger::init() {
        error!("Could not initialize logging: {}", e);
        process::exit(1);
    }

    let cfg = match Config::from_file(&cli.config_path()) {
        Ok(c) => c,
        Err(e) => {
            error!("Could not load configuration file: {}", e);
            process::exit(1);
        }
    };


    let connection = match DbConnection::connect(cfg.db().user(), cfg.db().passwd(),
                                                 cfg.db().db_name(), cfg.db().host(), cfg.db().port()) {
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

    let f_handles = feeder::start_feeders(
        &feed_sendr,
        cfg.redis().host(),
        cfg.redis().port(),
        cfg.workers().num_feeders()
    );

    let p_handles = processing::start_processors(
        &feed_recvr,
        &load_sendr,
        cfg.yara_rule_dir(),
        cfg.workers().num_processors() as usize
    );

    let l_handles = database::start_loaders(&load_recvr, db_loader, cfg.workers().num_loaders());

    // Feeders are the first threads to finish in the event of a graceful shutdown
    for handle in f_handles {
        handle.join().unwrap();
    }

    // Dropping the sender will gracefully close the receiver's end as well
    // and as such make all processor threads return
    drop(feed_sendr);

    // It is important to wait for all processor threads to join cleanly before
    // dropping the loader sender. If we drop both senders together, processor threads
    // that have events left in their queue will panic when they try to send matching ones
    // to the loader through the load channel
    for handle in p_handles {
        if let Ok(res) = handle.join() {
            if let Err(e) = res {
                error!("Error in processor: {}", e)
            }
        }
    }

    drop(load_sendr);

    for handle in l_handles {
        // We don't really care how loader threads exited
        handle.join().unwrap();
    }
}
