use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Once;

use nix::sys::signal;
use sysrepo::yang::data::DataNodeRef;
use sysrepo::yang::schema::{DataValue, SchemaNodeKind};
use sysrepo::Datastore;

#[allow(dead_code)]
pub fn datastore_to_str(ds: &Datastore) -> &str {
    match ds {
        Datastore::Startup => "startup",
        Datastore::Running => "running",
        Datastore::Candidate => "candidate",
        Datastore::Operational => "operational",
    }
}

#[allow(dead_code)]
pub fn str_to_datastore(s: &str) -> std::result::Result<Datastore, ()> {
    match s {
        "startup" => Ok(Datastore::Startup),
        "running" => Ok(Datastore::Running),
        "candidate" => Ok(Datastore::Candidate),
        "operational" => Ok(Datastore::Operational),
        _ => Err(()),
    }
}

#[allow(dead_code)]
pub fn print_node(node: DataNodeRef) {
    let value = match node.value() {
        Some(DataValue::Bool(i)) => i.to_string(),
        Some(DataValue::Int8(i)) => i.to_string(),
        Some(DataValue::Int16(i)) => i.to_string(),
        Some(DataValue::Int32(i)) => i.to_string(),
        Some(DataValue::Int64(i)) => i.to_string(),
        Some(DataValue::Uint8(i)) => i.to_string(),
        Some(DataValue::Uint16(i)) => i.to_string(),
        Some(DataValue::Uint32(i)) => i.to_string(),
        Some(DataValue::Uint64(i)) => i.to_string(),
        Some(DataValue::Other(s)) => s,
        Some(DataValue::Empty) => "(empty leaf)".to_owned(),
        None => match node.schema().kind() {
            SchemaNodeKind::Container => "(container)",
            SchemaNodeKind::List => "(list instance)",
            _ => "(unprintable)",
        }
        .to_owned(),
    };

    let default = if node.is_default() { " [default]" } else { "" };

    println!("{} {}{}", node.path(), value, default);
}

#[allow(dead_code)]
static SIGTSTP_ONCE: Once = Once::new();
#[allow(dead_code)]
static SIGINT_CAUGHT: AtomicUsize = AtomicUsize::new(0);

#[allow(dead_code)]
extern "C" fn sigint_handler(_: i32) {
    SIGINT_CAUGHT.fetch_add(1, Ordering::SeqCst);
}

#[allow(dead_code)]
pub fn is_sigint_caught() -> bool {
    SIGINT_CAUGHT.load(Ordering::SeqCst) > 0
}

#[allow(dead_code)]
pub fn signal_init() {
    SIGTSTP_ONCE.call_once(|| unsafe {
        let sa = signal::SigAction::new(
            signal::SigHandler::Handler(sigint_handler),
            signal::SaFlags::empty(),
            signal::SigSet::empty(),
        );
        let _ = signal::sigaction(signal::SIGINT, &sa);
    });
}
