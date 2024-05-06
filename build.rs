use chrono::Local;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use toml_edit::{value, DocumentMut};

fn main() {
    // if env::var("PROFILE").unwrap_or_default() == "debug" {
    //     println!("cargo:rerun-if-changed=build.rs");
    // }
    // let filename = "Cargo.toml";
    // let contents = fs::read_to_string(filename).unwrap();
    // let mut doc = contents.parse::<DocumentMut>().unwrap();

    // let date_version_key = Local::now().format("%Y.%-m.%-d").to_string();
    // let default_version = format!("{}-1", date_version_key);
    // let current_version = doc["package"]["version"]
    //     .as_str()
    //     .map(String::from)
    //     .unwrap_or(default_version);
    // let mut parts: Vec<_> = current_version.split('-').map(String::from).collect();

    // if parts[0] == date_version_key {
    //     if let Ok(build_number) = parts[1].parse::<i32>() {
    //         parts[1] = (build_number + 1).to_string();
    //     }
    // } else {
    //     parts = vec![date_version_key, "1".to_string()];
    // }

    // let new_version = parts.join("-");
    // doc["package"]["version"] = value(new_version.clone());
    // println!("cargo:warning=Version updated to {}", new_version);

    // Write the updated document back to Cargo.toml
    // fs::write(filename, doc.to_string()).unwrap();

    //**Copy Bibles
    let out_path = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));

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
