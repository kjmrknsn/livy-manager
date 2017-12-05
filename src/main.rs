extern crate livy_manager;

use livy_manager::server;

fn main() {
    server::run().unwrap()
}
