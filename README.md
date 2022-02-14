About hecdss
===
This library contains bindings to [HEC-DSS](http://www.hec.usace.army.mil/software/hec-dssvue/) in Rust. The included sys crate hecdss-sys automatically generates low-level C interface to Rust (FFI). The FFI is then used to create safe Rust APIs to HEC-DSS. This library is a work in progress. There is also a Python binding [pydsstools](https://github.com/gyanz/pydsstools) for HEC-DSS.

Usage
===

Install the [rustup](https://www.rust-lang.org/tools/install) toolchain.

Command to build the library from source:
```
cargo build
```

Command to build and run the included tests:
```
cargo test -- --nocapture
```

The build process involves linkage with HEC-DSS C and Fortran libraies. The included cmd_intel_environ.bat can be used in Windows 10 that has Microsoft Visual Studio and Intel OneApi.


Contributing
===
All contributions, bug reports, bug fixes, documentation improvements, enhancements and ideas are welcome.
Feel free to ask questions on my [email](mailto:gyanBasyalz@gmail.com).


License
===
This program is a free software: you can modify and/or redistribute it under [LICENSE](LICENSE-APACHE) license. 
