use bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-lib=heclib");
    println!("cargo:rustc-link-search=hecdss-sys/dss7/win64");
    println!("cargo:rerun-if-changed=dss7/headers/heclib.h");
    let bindings = bindgen::Builder::default()
        .header("dss7/headers/heclib.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
