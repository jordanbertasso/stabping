use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::process::ExitStatus;

static asset_files: &'static [&'static str] = &[
    "node_modules/dygraphs/dygraph-combined.js",
    "node_modules/vue/dist/vue.min.js",
    "app.js",
    "app.css",
    "index.html",
];

fn main() {
    let out_dir_str = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir_str);
    let proj_dir = env::current_dir().unwrap();
    let client_dir = proj_dir.join("client");

    match Command::new("npm")
                  .arg("install")
                  .current_dir(&client_dir).status() {
        Ok(s) if s.success() => (),
        _ => panic!("'npm install' of dependencies failed.")
    }

    let assets_out_dir = out_dir.join("assets");
    fs::create_dir_all(&assets_out_dir).unwrap();

    for asset_source in asset_files {
        let source_path = client_dir.join(asset_source);
        let asset_filename = source_path.file_name().unwrap();
        let target_path = assets_out_dir.join(asset_filename);
        fs::copy(&source_path, &target_path).unwrap();
    }
}
