extern crate coastguard;

use std::time::Duration;
use std::sync::mpsc::channel;

fn main() {
    let monitor = coastguard::Monitor {
        name: "meetup",
        url: "http://www.meetup.com/",
        interval: Duration::from_secs(5),
        timeout: Duration::from_secs(1)
    };
    let (tx, _) = channel();
    monitor.watch(tx);
}
