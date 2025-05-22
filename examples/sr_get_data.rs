/// An example of an application that gets values.
///
/// Adapted from `sysrepo` example rs_get_items_example.c`.

#[path = "../example_utils.rs"]
mod utils;

use std::env;

use sysrepo::*;
use yang::data::{Data, DataFormat, DataPrinterFlags};

use utils::*;

fn main() -> std::result::Result<(), ()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 && args.len() != 3 {
        println!(
            "Usage: {} <xpath-to-get> [startup/running/operational/candidate]",
            args[0]
        );
        return Err(());
    }

    let xpath = args[1].clone();
    let mut ds = Datastore::Running;

    if let Some(arg) = args.get(2) {
        if let Ok(datastore) = str_to_datastore(arg) {
            ds = datastore;
        } else {
            println!("Invalid datastore {}.", args[2]);
            return Err(());
        }
    }

    println!(
        "Application will get \"{}\" from \"{}\" datastore.",
        xpath,
        datastore_to_str(&ds),
    );

    // Turn logging on.
    set_stderr_log_level(LogLevel::Warn);

    // Connect to sysrepo.
    let connection = Connection::new(Default::default()).map_err(|_| ())?;

    // Start session.
    let session = connection.start_session(ds).map_err(|_| ())?;

    // Get the data.
    let data = session
        .get_data(&xpath, None, Default::default(), Default::default())
        .expect("Failed to get data");

    // Print data tree in the XML format.
    data.tree()
        .print_file(
            std::io::stdout(),
            DataFormat::XML,
            DataPrinterFlags::WD_ALL | DataPrinterFlags::WITH_SIBLINGS,
        )
        .expect("Failed to print data tree");

    Ok(())
}
