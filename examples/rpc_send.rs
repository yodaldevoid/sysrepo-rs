/// An example of an application that sends an RPC.
///
/// Adapted from `sysrepo` example `rpc_send_example.c`.

#[path = "../example_utils.rs"]
mod utils;

use std::env;

use sysrepo::*;
use yang::data::DataTree;

use utils::print_node;

fn main() -> std::result::Result<(), ()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Usage: {} <rpc-path>", args[0]);
        return Err(());
    }

    let path = args[1].clone();

    println!("Application will send RPC \"{}\" notification.", path);

    // Turn logging on.
    log_stderr(LogLevel::Warn);

    // Connect to sysrepo.
    let connection = Connection::new(Default::default()).map_err(|_| ())?;
    let ctx = connection.get_context().unwrap();

    // Start session.
    let mut session = connection
        .start_session(Datastore::Running)
        .map_err(|_| ())?;

    // Send the RPC.
    let mut rpc = DataTree::new(&ctx);
    if let Err(_) = rpc.new_path(&path, None, false) {
        println!("Creating RPC \"{}\" failed.", path);
        return Err(());
    }
    let data = session.rpc_send(rpc, Default::default()).map_err(|_| ())?;

    println!("\n ========== RECEIVED OUTPUT: ==========\n");
    for node in data.tree().traverse() {
        print_node(node);
    }

    Ok(())
}
