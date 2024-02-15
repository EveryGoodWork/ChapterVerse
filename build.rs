use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    // Get the OUT_DIR environment variable which is the build location for this component.
    let out_path = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
    // Define the source directory for the bibe CSV files
    let src_dir = Path::new("bibles");

    // Navigate two levels up from OUT_DIR to target the root of the build directory
    let target_dir = out_path
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .expect("Failed to resolve two levels up from OUT_DIR");

    // Specify the subdirectory where the CSV files should be placed
    let dest_dir = target_dir.join("bibles");
    if let Err(e) = fs::create_dir_all(&dest_dir) {
        println!(
            "cargo:warning=Failed to create destination directory: {:?}",
            e
        );
    } else {
        println!("cargo:warning=CSV files will be copied to: {:?}", dest_dir);
    }

    // Copy each CSV file from the source to the target directory
    for entry in fs::read_dir(src_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("csv") {
            let dest_path = dest_dir.join(entry.file_name());
            if let Err(e) = fs::copy(path, dest_path) {
                println!("cargo:warning=Failed to copy file: {:?}", e);
            }
        }
    }
}
