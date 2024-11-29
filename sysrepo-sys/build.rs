extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-lib=sysrepo");
    println!("cargo:rustc-link-lib=yang");

    let yang2 = env::var("CARGO_FEATURE_YANG2").is_ok();
    let yang3 = env::var("CARGO_FEATURE_YANG3").is_ok();
    let yang_lib = match [yang2, yang3] {
        [true, false] => "use libyang2_sys::*;",
        [false, true] => "use libyang3_sys::*;",
        _ => panic!("One and only one of the yang* features must be set"),
    };

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .derive_default(true)
        .size_t_is_usize(false)
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        .raw_line(yang_lib)
        .raw_line("use libc::size_t;")
        .allowlist_item("sr_.*")
        .allowlist_item("srplg_.*")
        .allowlist_item("SR_.*")
        .allowlist_item("SRP_.*")
        .allowlist_item("SRPLG_.*")
        .allowlist_recursively(false)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
