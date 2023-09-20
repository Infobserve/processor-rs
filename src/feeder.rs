use log::{info, error};
use std::thread::{self, JoinHandle};

use crossbeam_channel::Sender;
use redis::{Client, Commands, Connection};
use anyhow::Result;

use crate::entities::Event;

/// Spawns `num_feeders` threads. Each thread listens for events through redis. Whenever an event is fetched,
/// a message is written in the sender end of a crossbeam channel (normally, a processing thread is listening
/// on the receiving end of that)
/// 
/// # Arguments
/// 
/// * sendr - The write-end of a crossbeam channel. All events fetched from redis will be written there.
///           If a quit message is received instead of an event, then this sender is dropped, effectively
///           unblocking all threads listening to it.
/// * host - Redis host
/// * port - Redis port
/// * num_feeders - The amount of feeder threads to spawn
/// 
/// # Return
/// A vector of join handles that can be used to join the threads. Threads will exit their loops only
/// if a quit command is received from Redis.
/// 
/// # Example
/// ```
/// use feeder::start_feeders;
/// 
/// let (proc_sendr, proc_receiver) = crossbeam_channel::unbounded();
///
/// let handles: Vec<JoinHandle<()>> = start_feeders(&proc_sendr, "localhost", 6379, 2);
///
/// assert_eq!(handles.len(), 2);
/// // for msg in proc_receiver {
/// //     println!("Received event!");
/// // }
///
/// for handle in handles {
///     handle.join().unwrap();
/// }
/// ```
pub fn start_feeders(sendr: &Sender<Event>, host: &str, port: u16, num_feeders: i32) -> Vec<JoinHandle<()>> {
    let mut threads = Vec::with_capacity(num_feeders as usize);

    for _ in 0..num_feeders {
        let mut feeder = Feeder::connect(&host, port).expect(&format!("redis connection @redis://{}:{}", host, port));
        let sendr_copy = Sender::clone(sendr);
        threads.push(
            thread::spawn(move || {
                if let Err(e) = feeder.listen(&sendr_copy) {
                    error!("Feeder encountered an error!: {}", e);
                    return;
                }
            })
        );
    }

    threads
}

struct Feeder {
    client: Client
}

impl Feeder {
    /// Opens a connection to a Redis server and retains a handle for it
    fn connect(host: &str, port: u16) -> Result<Self> {
        let client = Client::open(format!("redis://{}:{}/", host, port))?;

        Ok(Self { client })
    }

    /// Continuously listens for events from Redis. Whenever an event is encountered, it is written
    /// in `sendr`
    fn listen(&mut self, sendr: &Sender<Event>) -> Result<()> {
        let mut conn = self.client.get_connection()?;

        loop {
            let msg = match self.pop_msg(&mut conn) {
                Ok(m) => m,
                Err(e) => {
                    error!("Could not pop event from redis queue: {}", e);
                    continue;
                }
            };
            
            info!("New message in {}", msg.name);

            let payload = msg.payload;

            if &payload == "QUIT" {
                break;
            }

            match Event::from_json_str(&payload) {
                Ok(e) => {
                    if let Err(e) = sendr.send(e) {
                        error!("Could not send event to processor: {}", e);
                    }
                },
                Err(e) => error!("Could not deserialize message from redis: msg: {}, error: {}", payload, e)
            }
        }

        Ok(())
    }

    fn pop_msg(&self, conn: &mut Connection) -> Result<Message> {
        let msg: Vec<String> = conn.blpop("events", 0)?;

        Ok(Message {
            name: msg[0].to_owned(),
            payload: msg[1].to_owned()
        })
    }
}

struct Message {
    name: String,
    payload: String
}