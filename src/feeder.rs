use log::{info, error};
use std::thread::{self, JoinHandle};

use crossbeam_channel::Sender;
use redis::{Connection, Client};
use anyhow::Result;

use crate::entities::Event;

pub fn start_feeders(sendr: &Sender<Event>, host: &str, port: u16, num_feeders: i32) -> Vec<JoinHandle<()>> {
    let mut threads = Vec::with_capacity(num_feeders as usize);

    for _ in 0..num_feeders {
        let mut feeder = Feeder::connect(&host, port).unwrap();
        let sendr_copy = Sender::clone(sendr);
        threads.push(
            thread::spawn(move || {
                if let Err(e) = feeder.listen(&sendr_copy) {
                    error!("Feeder hit an error!: {}", e);
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
    fn connect(host: &str, port: u16) -> Result<Self> {
        let client = Client::open(format!("redis://{}:{}/", host, port))?;

        Ok(Self { client })
    }

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

            let payload = msg.payload;

            if &payload == "QUIT" {
                break;
            }

            info!("Got new event");

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
        let msg: Vec<String> = redis::cmd("BLPOP").arg("events").arg(0).cursor_arg(0).clone().iter(conn)?.collect();

        Ok(Message {
            name: msg[0].clone(),
            payload: msg[1].clone()
        })
    }
}

struct Message {
    name: String,
    payload: String
}