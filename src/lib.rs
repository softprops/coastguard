extern crate hyper;
extern crate time;

use std::sync::mpsc::Sender;
use std::time::Duration;
use std::thread::sleep;
use hyper::Client;
use hyper::status::StatusCode;

pub struct Log {
    entries: Vec<Entry>
}

impl Log {
    pub fn add(&mut self, entry: Entry) {
        self.entries.push(entry);
    }
}

pub enum Result {
    Ok,
    Anomaly(StatusCode),
    Timeout
}

pub struct Entry {
    pub millis: i64,
    pub result: Result
}

pub struct Monitor<'a> {
    pub name: &'a str,
    pub url: &'a str,
    pub interval: Duration,
    pub timeout: Duration
}

impl <'a> Monitor <'a> {
    pub fn watch(&self, tx: Sender<Entry>) {
        let mut client = Client::new();
        client.set_read_timeout(Some(self.timeout));
        loop {
            let moment = time::now();
            let result = client.head(self.url).send();
            let elapsed = time::now() - moment;
            match result {
                Ok(response) => {
                    let entry = match response.status {
                        StatusCode::Ok => Entry {
                            millis: elapsed.num_milliseconds(),
                            result: Result::Ok
                        },
                        unexpected => Entry {
                            millis: elapsed.num_milliseconds(),
                            result: Result::Anomaly(unexpected)
                        }
                    };
                    let _ = tx.send(entry);
                },
                Err(_) => {
                    let _ = tx.send(
                        Entry {
                            millis: elapsed.num_milliseconds(),
                            result: Result::Timeout
                        });
                }
            }

            sleep(self.interval)
        }
    }
}
