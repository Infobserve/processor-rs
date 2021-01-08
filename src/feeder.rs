use log::{info, warn, error};
use std::thread::{self, JoinHandle};

use crossbeam_channel::Sender;
use redis::Client;
use anyhow::Result;

use crate::entities::Event;

pub fn start_feeders(sendr: &Sender<Event>, host: &str, port: u16, num_feeders: i32) -> Vec<JoinHandle<()>> {
    let mut threads = Vec::with_capacity(num_feeders as usize);

    for _ in 0..num_feeders {
        let mut feeder = Feeder::connect(&host, port).unwrap();
        let sendr_copy = Sender::clone(sendr);
        threads.push(
            thread::spawn(move || {
                feeder.listen(&sendr_copy).unwrap();
            })
        );
    }

    threads
}

struct Feeder {
    client: Client
}

impl Feeder {
    fn connect(host: &str, port: u16) -> Result<Self> {
        let client = Client::open(format!("redis://{}:{}/", host, port))?;

        Ok(Self { client })
    }

    fn listen(&mut self, sendr: &Sender<Event>) -> Result<()> {
        let mut con = self.client.get_connection()?;
        let mut sub_handle = con.as_pubsub();

        for channel in vec!["events", "cmd"] {
            sub_handle.subscribe(channel)?;
        }

        loop {
            let msg = sub_handle.get_message()?;
            let payload = msg.get_payload::<String>()?;

            match msg.get_channel_name() {
                "cmd" => {
                    if payload == String::from("quit") {
                        break;
                    } else {
                        warn!("Unknown message received at cmd channel: {}", payload);
                    }
                },
                "events" => {
                    info!("Got new event!");
                    match Event::from_json_str(&payload) {
                        Ok(e) => {
                            if let Err(e) = sendr.send(e) {
                                error!("Could not send event to processor: {}", e);
                            }
                        },
                        Err(e) => error!("Could not deserialize message from redis: msg: {}, error: {}", payload, e)

                    }
                },
                x => error!("Message received in unknown channel ({}): {}", x, payload)
            }
        }

        Ok(())
    }
}