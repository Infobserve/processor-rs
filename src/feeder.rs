use log::{info, warn, error};
use std::{sync::Arc, thread};
use std::thread::JoinHandle;

use redis::{Client, Connection, PubSub};
use anyhow::Result;

use crate::errors::NotSubscribedError;

pub fn start_feeders(host: &str, port: u16, num_feeders: i32) -> Vec<JoinHandle<()>> {
    let mut threads = Vec::with_capacity(num_feeders as usize);
    let host_arc = Arc::new(host.to_owned());

    for _ in 0..num_feeders {
        threads.push(listen(&host_arc, port));
    }

    threads
}

fn listen(host_arc: &Arc<String>, port: u16) -> JoinHandle<()> {
    let host = Arc::clone(&host_arc);

    thread::spawn(move || {
        let mut feeder = Feeder::connect(&host, port).unwrap();
        feeder.listen().unwrap();
    })
}

struct Feeder {
    client: Client
}

impl Feeder {
    fn connect(host: &str, port: u16) -> Result<Self> {
        let client = Client::open(format!("redis://{}:{}/", host, port))?;

        Ok(Self { client })
    }

    fn listen(&mut self) -> Result<()> {
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
                        warn!("Unknown message @ cmd channel: {}", payload);
                    }
                },
                "events" => {
                    info!("Got new event!");
                },
                x => error!("Message received in unknown channel ({}): {}", x, payload)
            }
        }

        Ok(())
    }
}