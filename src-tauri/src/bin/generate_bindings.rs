use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output_path = manifest_dir.join("../src/bindings.ts");

    if let Err(err) = wereply_lib::bindings::export_typescript_bindings(&output_path) {
        eprintln!("failed to generate bindings: {err}");
        std::process::exit(1);
    }
}
