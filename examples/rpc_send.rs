//
// Sysrepo-examples.
//   rpc_send
//

#[path = "../example_utils.rs"]
mod utils;

use std::env;

use sysrepo::*;
use yang::data::DataTree;

use utils::print_node;

/// Show help.
fn print_help(program: &str) {
    println!(
        "Usage: {} <notification-path> [<node-to-set> <node-value>]",
        program
    );
}

/// Main.
fn main() {
    if run() {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}

fn run() -> bool {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    if args.len() != 2 {
        print_help(&program);
        return false;
    }

    let path = args[1].clone();

    println!(r#"Application will send RPC "{}" notification."#, path);

    // Turn logging on.
    log_stderr(LogLevel::Warn);

    // Connect to sysrepo.
    let sr = match Connection::new(Default::default()) {
        Ok(sr) => sr,
        Err(_) => return false,
    };
    let ctx = sr.get_context().unwrap();

    // Start session.
    let mut sess = match sr.start_session(Datastore::Running) {
        Ok(sess) => sess,
        Err(_) => return false,
    };

    // Send the RPC.
    let mut rpc = DataTree::new(&ctx);
    if let Err(_) = rpc.new_path(&path, None, false) {
        println!("Creating RPC \"{}\" failed.", path);
        return false;
    }
    match sess.rpc_send(rpc, Default::default()) {
        Ok(data) => {
            for node in data.tree().traverse() {
                print_node(node);
            }
        }
        Err(_) => return false,
    };

    true
}
