use bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    let os = env::var("CARGO_CFG_TARGET_OS");
    println!("cargo:rustc-link-lib=heclib");
    match os.as_ref().map(|x|&**x) {
        //Ok("linux") => println!("cargo:rustc-link-search=hecdss-sys/dss7/linux64"),
        Ok("linux") => println!("cargo:rustc-link-search={}/dss7/linux64",std::env::current_dir().unwrap().display()),
        //Ok("windows") => println!("cargo:rustc-link-search=hecdss-sys/dss7/win64"),
        Ok("windows") => println!("cargo:rustc-link-search={}/dss7/win64",std::env::current_dir().unwrap().display()),
        _ => panic!("Operating system not supported")
    };
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
