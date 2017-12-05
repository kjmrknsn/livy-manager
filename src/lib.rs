//! # livy_manager
//! Web UI for Managing Apache Livy Sessions

extern crate argparse;

/// Command-line arguments
pub mod cmd_args;
/// HTTP server
pub mod server;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
