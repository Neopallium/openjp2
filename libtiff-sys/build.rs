use std::env;
use std::path::PathBuf;

fn main() {
  // Link against libtiff
  pkg_config::probe_library("libtiff-4").unwrap();

  // Generate bindings
  let bindings = bindgen::Builder::default()
    .header("wrapper.h")
    .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
    // Allow all TIFF related items
    .allowlist_type("TIFF.*")
    .allowlist_function("TIFF.*")
    .allowlist_var("TIFF.*")
    .allowlist_var("PHOTOMETRIC.*")
    .allowlist_var("PLANARCONFIG_.*")
    .allowlist_var("ORIENTATION_.*")
    .allowlist_var(".*FORMAT_.*")
    .allowlist_var(".*TAG_.*")
    // Additional common libtiff functions
    .allowlist_function("_TIFF.*")
    .allowlist_function(".*tiff.*")
    .allowlist_var("_TIFF.*")
    // System types that libtiff depends on
    .allowlist_type("tsize_t")
    .allowlist_type("tdata_t")
    .allowlist_type("toff_t")
    .allowlist_type("thandle_t")
    // Generate bindings for constants
    .generate_comments(true)
    .generate()
    .expect("Unable to generate bindings");

  let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
  bindings
    .write_to_file(out_path.join("bindings.rs"))
    .expect("Failed to write bindings");
}
