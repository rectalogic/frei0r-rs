use std::{env, path::PathBuf};

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    #[cfg(feature = "bindgen")]
    bindgen::Builder::default()
        .header("frei0r.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_item("f0r_.*")
        .allowlist_item("F0R_.*")
        .allowlist_item("FREI0R_.*")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    #[cfg(not(feature = "bindgen"))]
    {
        use std::fs;

        let crate_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        fs::copy(crate_path.join("bindings.rs"), out_path.join("bindings.rs"))
            .expect("Couldn't find pregenerated bindings!");
    }
}
