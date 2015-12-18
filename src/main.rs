extern crate coastguard;
extern crate rustc_serialize;

use rustc_serialize::json;
use std::fs::File;
use std::io::Read;

fn main() {
    let mut f = File::open("coastguard.json").unwrap();
    let mut s = String::new();
    let _ = f.read_to_string(&mut s);
    let config = json::decode::<coastguard::Config>(&s).unwrap();
    println!("{:?}", config.watches())
}
