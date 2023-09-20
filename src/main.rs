//! This binary crate handles the processing part of the [infobserve project](https://github.com/Infobserve/infobserve).
//! It's split into 3 distinct components:
//! 1. [Feeder](crate::feeder): Pops messages from redis. Each message (JSON format) represents an event, as fetched by
//!    the infobserve part (python). After fetching a message, it deserializes it into an [Event](crate::entities::Event) object
//!    and sends it for processing using the F-P (feeder-processor) crossbeam channel
//! 2. [Processor](crate::processing): Pops events from the F-P crossbeam channel. Each event's contents
//!    are processed using the specified Yara rules. If an event matches any of the Yara rules, a
//!    [ProcessedEvent](crate::entities::ProcessedEvent) (which contains both the initial event as well as the matched
//!    parts) is pushed into the P-L (processor-loader) crossbeam channel
//! 3. [DbLoader](crate::database::DbLoader): Pops [ProcessedEvent](crate::entities::ProcessedEvent)s from the P-L
//!    crossbeam channel, splits them into normalized database entities
//!    ([Event](crate::entities::Event), [RuleMatch](crate::entities::RuleMatch), [AsciiMatch](crate::entities::AsciiMatch))
//!    and inserts them into the database.
//!
//! # Configuration
//!
//! * **workers**: A hash specifying the number of threads each worker type will use.
//!                Alternatively can be set to `auto` in which case the system's logical threads will be distributed
//!                automatically among the workers as such: processor workers will be assigned 50% of the available threads,
//!                and feeder & loader workers will be assigned 25% each. Keep in mind that all these values are `floor`ed so
//!                not all available threads will be necessarily used
//!     * **feeders**: Number of feeder threads. Default: `1`
//!     * **processors**: Number of processor threads. Default: `1`
//!     * **loaders**: Number of loader threads. Default: `1`
//! * **yara_rule_dir**: Path to the root direction which contains the Yara rules (`.yar` extension).
//!                      Default: `./yara-rules/`
//! * **database**: A hash specifying how to connect to the postgres server
//!     * **user**: Default: `postgres`
//!     * **passwd**: This can either be set here or in the `INFOBSERVE_POSTGRES_PASSWD` environment
//!                   variable, with the former taking precedence. Default: `infobserve`
//!     * **db_name**: The database name. Default: `infobserve`
//!     * **host**: Default: `localhost`
//!     * **port**: Default: `5432`
//!
//! ## Example configuration:
//! ```yaml
//! workers:
//!     processors: 5
//!     feeders: 2
//! yara_rule_dir: ./yara
//! database:
//!     password: 54infobserve32
//!     db_name: public
//! ```
//!
//! Note: A configuration template can be found in [`config.tpl.yaml`](https://github.com/Infobserve/processor-rs/blob/main/config.tpl.yaml)
//!
//! # Execution:
//! Simply run `cargo run` (or `cargo run --release` if you've got time to kill). The feeder workers will begin
//! popping from redis' `events` list. They won't pop anything however, until a
//! [producer](https://github.com/Infobserve/infobserve#working-with-processor-rs) comes into play
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
    let cli: Cli = Cli::parse_args();

    if let Err(e) = logger::init() {
        error!("Could not initialize logger: {}", e);
        process::exit(1);
    }

    let cfg = match Config::from_file(cli.config_path()) {
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
