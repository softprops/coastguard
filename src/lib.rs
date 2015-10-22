extern crate hyper;
extern crate time;

use hyper::server::{Handler, Server, Request, Response};
use std::sync::Mutex;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;
use std::thread::{sleep, sleep_ms};
use hyper::Client;

pub struct Monitor<'a> {
    pub name: &'a str,
    pub url: &'a str,
    pub interval: Duration,
    pub timeout: Duration
}

impl <'a> Monitor <'a> {
    pub fn watch(&self, tx: Sender<&str>) {
        let mut client = Client::new();
        client.set_read_timeout(Some(self.timeout));
        loop {
            let moment = time::now();
            match client.head(self.url).send() {
                Ok(_) => {
                    let elapsed = time::now() - moment;
                    let std_duration = Duration::from_millis(
                        elapsed.num_milliseconds() as u64
                            );
                    println!("{:?}", elapsed.num_milliseconds());
                    tx.send("OK");
                },
                Err(_) => {
                    tx.send("Fail");
                }
            }

            sleep(self.interval)
        }
    }
}
