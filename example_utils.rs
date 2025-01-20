use std::ffi::CStr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Once;

use nix::sys::signal;
use sysrepo::yang::data::DataNodeRef;
use sysrepo::yang::schema::{DataValue, SchemaNodeKind};
use sysrepo_sys::*;

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
pub fn print_val(value: &sr_val_t) {
    let xpath: &CStr = unsafe { CStr::from_ptr(value.xpath) };

    print!("{} ", xpath.to_str().unwrap());

    let v = match value.type_ {
        sr_val_type_t::SR_CONTAINER_T | sr_val_type_t::SR_CONTAINER_PRESENCE_T => String::from("(container)"),
        sr_val_type_t::SR_LIST_T => String::from("(list instance)"),
        sr_val_type_t::SR_STRING_T => {
            let string_val = unsafe { CStr::from_ptr(value.data.string_val) };
            format!("= {}", string_val.to_str().unwrap())
        }
        sr_val_type_t::SR_BOOL_T => {
            // https://github.com/sysrepo/sysrepo/blob/2af422e1aecc8997d069c50f65d213da31b8a7e2/src/common.c#L4651
            let bool_val = unsafe { value.data.bool_val };
            format!("= {}", if bool_val == 1 { true } else { false })
        }
        sr_val_type_t::SR_DECIMAL64_T => {
            let dec64_val = unsafe { value.data.decimal64_val as f64 };
            format!("= {}", dec64_val)
        }
        sr_val_type_t::SR_INT8_T => {
            let int8_val = unsafe { value.data.int8_val as i8 };
            format!("= {}", int8_val)
        }
        sr_val_type_t::SR_INT16_T => {
            let int16_val = unsafe { value.data.int16_val as i16 };
            format!("= {}", int16_val)
        }
        sr_val_type_t::SR_INT32_T => {
            let int32_val = unsafe { value.data.int32_val as i32 };
            format!("= {}", int32_val)
        }
        sr_val_type_t::SR_INT64_T => {
            let int64_val = unsafe { value.data.int64_val as i64 };
            format!("= {}", int64_val)
        }
        sr_val_type_t::SR_UINT8_T => {
            let uint8_val = unsafe { value.data.uint8_val as u8 };
            format!("= {}", uint8_val)
        }
        sr_val_type_t::SR_UINT16_T => {
            let uint16_val = unsafe { value.data.uint16_val as u16 };
            format!("= {}", uint16_val)
        }
        sr_val_type_t::SR_UINT32_T => {
            let uint32_val = unsafe { value.data.uint32_val as u32 };
            format!("= {}", uint32_val)
        }
        sr_val_type_t::SR_UINT64_T => {
            let uint64_val = unsafe { value.data.uint64_val as u64 };
            format!("= {}", uint64_val)
        }
        sr_val_type_t::SR_IDENTITYREF_T => {
            let identityref_val = unsafe { CStr::from_ptr(value.data.identityref_val) };
            format!("= {}", identityref_val.to_str().unwrap())
        }
        sr_val_type_t::SR_INSTANCEID_T => {
            let instanceid_val = unsafe { CStr::from_ptr(value.data.instanceid_val) };
            format!("= {}", instanceid_val.to_str().unwrap())
        }
        sr_val_type_t::SR_BITS_T => {
            let bits_val = unsafe { CStr::from_ptr(value.data.bits_val) };
            format!("= {}", bits_val.to_str().unwrap())
        }
        sr_val_type_t::SR_BINARY_T => {
            let binary_val = unsafe { CStr::from_ptr(value.data.binary_val) };
            format!("= {}", binary_val.to_str().unwrap())
        }
        sr_val_type_t::SR_ENUM_T => {
            let enum_val = unsafe { CStr::from_ptr(value.data.enum_val) };
            format!("= {}", enum_val.to_str().unwrap())
        }
        sr_val_type_t::SR_LEAF_EMPTY_T => String::from("(empty leaf)"),
        _ => String::from("(unprintable)"),
    };

    match value.type_ {
        sr_val_type_t::SR_UNKNOWN_T
        | sr_val_type_t::SR_CONTAINER_T
        | sr_val_type_t::SR_CONTAINER_PRESENCE_T
        | sr_val_type_t::SR_LIST_T
        | sr_val_type_t::SR_LEAF_EMPTY_T => println!("{}", v),
        _ => println!("{}{}", v, if value.dflt == 1 { " [default]" } else { "" }),
    }
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
