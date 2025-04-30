/// An example of an application handling changes.
///
/// Adapted from `sysrepo` example `application_changes_example.c`.

#[path = "../example_utils.rs"]
mod utils;

use std::env;
use std::thread;
use std::time;

use sysrepo::*;
use yang::data::DataTree;

use utils::*;

fn print_change(node: &DataTree, oper: ChangeOperation) {
    let node = node.reference().unwrap();
    match oper {
        ChangeOperation::Created
        | ChangeOperation::CreatedLeafListUserOrdered { .. }
        | ChangeOperation::CreatedListUserOrdered { .. } => {
            print!("CREATED: ");
            print_node(node);
        }
        ChangeOperation::Deleted => {
            print!("DELETED: ");
            print_node(node);
        }
        ChangeOperation::Modified {
            previous_value,
            previous_default,
        } => {
            let default = if previous_default { " [default]" } else { "" };
            print!("MODIFIED: {}{} to ", previous_value, default);
            print_node(node);
        }
        ChangeOperation::MovedLeafListUserOrdered {
            previous_value: previous,
        }
        | ChangeOperation::MovedListUserOrdered {
            previous_key: previous,
        } => {
            print!("MOVED: ");
            print_node(node);
            if previous.is_empty() {
                println!(" to the beginning");
            } else {
                println!(" after {}", previous);
            }
        }
    }
}

fn print_current_config(sess: &Session, mod_name: &str) -> Result<()> {
    let xpath = format!("/{}:*//.", mod_name);

    let data = sess.get_data(&xpath, None, Default::default(), Default::default())?;
    for node in data.tree().traverse() {
        print_node(node);
    }

    Ok(())
}

fn main() -> std::result::Result<(), ()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args.len() > 4 {
        println!(
            "Usage: {} <module-to-subscribe> [<xpath-to-subscribe>] [startup/running/operational/candidate]",
            args[0],
        );
        return Err(());
    }

    let mod_name = args[1].clone();
    let mut xpath: Option<String> = None;
    let mut ds = Datastore::Running;

    if let Some(arg) = args.get(2) {
        if let Ok(datastore) = str_to_datastore(arg) {
            ds = datastore;
        } else {
            xpath = Some(arg.clone());
        }
    }
    if let Some(arg) = args.get(3) {
        if let Ok(datastore) = str_to_datastore(arg) {
            ds = datastore;
        } else {
            println!("Invalid datastore {}", arg);
            return Err(());
        }
    }

    let xpath = xpath.as_ref().map(String::as_str);

    println!(
        "Application will watch for \"{}\" changes in \"{}\" datastore.",
        xpath.unwrap_or(&mod_name),
        datastore_to_str(&ds),
    );

    // Turn logging on.
    log_stderr(LogLevel::Warn);

    // Connect to sysrepo.
    let connection = Connection::new(Default::default()).map_err(|_| ())?;

    // Start session.
    let session = connection.start_session(ds).map_err(|_| ())?;

    // Read current config.
    println!("\n ========== READING RUNNING CONFIG: ==========\n");
    print_current_config(&session, &mod_name).map_err(|_| ())?;

    let module_change_cb = |session: &Session,
                            _sub_id: u32,
                            module_name: &str,
                            xpath: Option<&str>,
                            event: Event,
                            _request_id: u32| {
        println!(
            "\n\n ========== EVENT {} CHANGES: ====================================\n",
            event,
        );

        let path = if let Some(xpath) = xpath {
            format!("{}//.", xpath)
        } else {
            format!("/{}:*//.", module_name)
        };
        let changes = match session.get_changes_iter(&path) {
            Ok(iter) => iter,
            Err(_) => return Ok(()),
        };

        for change in &changes {
            match change {
                Ok((node, oper)) => print_change(&node, oper),
                Err(_) => return Ok(()),
            }
        }

        println!("\n ========== END OF CHANGES =======================================");

        if event == Event::Done {
            println!("\n\n ========== CONFIG HAS CHANGED, CURRENT RUNNING CONFIG: ==========\n");
            print_current_config(&session, module_name)?;
        }

        Ok(())
    };

    // Subscribe for changes in running config.
    let _subscription = session
        .new_module_change_subscription(&mod_name, xpath, module_change_cb, 0, Default::default())
        .map_err(|_| ())?;

    println!("\n\n ========== LISTENING FOR CHANGES ==========\n");

    signal_init();
    while !is_sigint_caught() {
        thread::sleep(time::Duration::from_secs(1));
    }

    println!("Application exit requested, exiting.");

    Ok(())
}
