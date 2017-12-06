//! # livy_manager
//! Web UI for Managing Apache Livy Sessions

extern crate argparse;
extern crate futures;
extern crate hyper;
extern crate ldap3;
extern crate livy;
extern crate percent_encoding;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate time;
extern crate toml;
extern crate uuid;

/// Command-line arguments
pub mod cmd_args;
/// Configuration for Livy Manager
pub mod config;
/// Frontend resources
pub mod frontend;
/// LDAP client
pub mod ldap;
/// Livy Manager
pub mod livy_manager;
/// HTTP server
pub mod server;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
