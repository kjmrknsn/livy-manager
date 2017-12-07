//! # livy_manager
//! Web UI for Managing Apache Livy Sessions

extern crate argparse;
extern crate iron;
extern crate iron_sessionstorage;
extern crate ldap3;
extern crate livy;
extern crate params;
extern crate persistent;
extern crate router;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate toml;

/// Command-line arguments
pub mod cmd_args;
/// Configuration for Livy Manager
pub mod config;
/// Frontend resources
pub mod frontend;
/// LDAP client
pub mod ldap;
/// HTTP server
pub mod server;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
