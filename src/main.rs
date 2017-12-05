extern crate livy_manager;

use livy_manager::server;
use std::env;

fn main() {
    server::run(env::args()).unwrap()
}
