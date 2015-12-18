extern crate hyper;
extern crate time;
extern crate rustc_serialize;

use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::time::Duration;
use std::thread::sleep;
use hyper::Client;
use hyper::status::StatusCode;

#[derive(Debug, RustcDecodable)]
pub struct WatchConfig {
    url: String,
    name: Option<String>,
    interval: Option<u64>,
    timeout: Option<usize>,
    threshold: Option<usize>
}

#[derive(Debug, RustcDecodable)]
pub struct NotifierConfig {
    name: String,
    config: Option<HashMap<String, String>>
}

#[derive(Debug, RustcDecodable)]
pub struct Config {
    watches: Vec<WatchConfig>,
    notifiers: Vec<NotifierConfig>
}

impl Config {
    pub fn watches(&self) -> Vec<Watch> {
        self.watches.iter().map(|m| {
            Watch::new(
                m.name.clone().unwrap_or(m.url.clone()),
                m.url.clone(),
                Duration::from_millis(m.interval.unwrap_or(10_000)),
                Duration::from_millis(m.interval.unwrap_or(5_000)),
                m.threshold.unwrap_or(1)
            )
        }).collect::<Vec<Watch>>()
    }

    pub fn notifiers(&self) -> Vec<Box<Notifier>> {
        self.notifiers.iter().filter_map(|n| {
            Notifier::new(
                n.name.clone(),
                n.config.clone().unwrap_or(HashMap::new()))
        }).collect::<Vec<Box<Notifier>>>()
    }
}

pub trait Notifier {
    fn notify(&self, watch: &Watch, log: &Log) -> ();
    fn close(&self) -> ();
}

struct Email;

impl Notifier for Email {
    fn notify(&self, watch: &Watch, log: &Log) {}
    fn close(&self) {}
}

struct PagerDuty;

impl Notifier for PagerDuty {
    fn notify(&self, watch: &Watch, log: &Log) {}
    fn close(&self) {}
}

impl Notifier {
    pub fn new<S>(name: S, config: HashMap<String, String>) -> Option<Box<Notifier>> where S: Into<String> {
        match name.into().as_ref() {
            "email" => Some(Box::new(Email)),
            "pagerduty" => Some(Box::new(PagerDuty)),
            _ => None
        }
    }
}

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

#[derive(Debug)]
pub struct Watch {
    pub name: String,
    pub url: String,
    pub interval: Duration,
    pub timeout: Duration,
    pub threshold: usize
}

impl Watch  {
    pub fn new<N, U>(name: N, url: U, interval: Duration, timeout: Duration, threshold: usize) -> Watch where N: Into<String>, U: Into<String> {
        Watch {
            name: name.into(),
            url: url.into(),
            interval: interval,
            timeout: timeout,
            threshold: threshold
        }
    }
    pub fn watch(&self, tx: Sender<Entry>) {
        let mut client = Client::new();
        client.set_read_timeout(Some(self.timeout));
        loop {
            let moment = time::now();
            let result = client.head(&self.url[..]).send();
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
