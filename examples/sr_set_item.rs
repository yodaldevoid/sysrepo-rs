/// An example of an application that sets a value.
///
/// Adapted from `sysrepo` example rs_set_item_example.c`.
use std::env;

use sysrepo::*;

fn main() -> std::result::Result<(), ()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("Usage: {} <x-path-to-set> <value-to-set>", args[0]);
        return Err(());
    }

    let xpath = args[1].clone();
    let value = args[2].clone();

    println!("Application will get \"{}\" to \"{}\".", xpath, value);

    // Turn logging on.
    log_stderr(LogLevel::Warn);

    // Connect to sysrepo.
    let connection = Connection::new(Default::default()).map_err(|_| ())?;

    // Start session.
    let mut session = connection
        .start_session(Datastore::Running)
        .map_err(|_| ())?;

    // Set the value.
    session
        .set_item_str(&xpath, &value, None, Default::default())
        .map_err(|_| ())?;

    // Apply the change.
    session.apply_changes(Default::default()).map_err(|_| ())?;

    Ok(())
}
