use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::process::ExitStatus;

static ASSET_FILES: &'static [&'static str] = &[
    "node_modules/dygraphs/dygraph-combined.js",
    "node_modules/vue/dist/vue.min.js",
    "app.js",
    "styles.css",
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

    let mut wahb = String::new();
    wahb.push_str("fn _webassets_handler_body<'a>(path: &'a str) -> Option<(&'static str, &'static str)> {\n");
    wahb.push_str("match path {\n");

    for asset_source in ASSET_FILES {
        let source_path = client_dir.join(asset_source);
        let asset_filename = source_path.file_name().unwrap();
        let target_path = assets_out_dir.join(asset_filename);
        let content_type = match target_path.extension().unwrap().to_str().unwrap() {
            "html" | "htm" => "text/html",
            "js" => "application/javascript",
            "css" => "text/css",
            _ => "text/plain"
        };
        let match_arm = format!("\"{}\" => Some((include_str!(\"{}\"), \"{}\")),\n",
                                asset_filename.to_str().unwrap(),
                                target_path.to_str().unwrap(),
                                content_type);
        wahb.push_str(&match_arm);
        fs::copy(&source_path, &target_path).unwrap();
    }

    wahb.push_str("_ => None\n");
    wahb.push_str("}\n");
    wahb.push_str("}\n");

    let wa_handler_body_path = out_dir.join("webassets_handler_body.rs");
    let mut wa_handler_body_file = File::create(&wa_handler_body_path).unwrap();
    wa_handler_body_file.write_all(wahb.as_bytes()).unwrap();
}
