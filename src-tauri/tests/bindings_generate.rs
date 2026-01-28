use std::path::PathBuf;

use wereply_lib::bindings::export_typescript_bindings;

fn temp_output_path() -> PathBuf {
    let mut path = std::env::temp_dir();
    let suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    path.push(format!("wereply_bindings_{suffix}.ts"));
    path
}

#[test]
fn generated_bindings_include_api_key_args() {
    let output = temp_output_path();

    export_typescript_bindings(&output).expect("export should succeed");

    let contents = std::fs::read_to_string(&output).expect("bindings file should exist");

    assert!(
        contents.contains("saveApiKey") && contents.contains("apiKey"),
        "bindings should include apiKey parameter for saveApiKey"
    );
    assert!(
        contents.contains("diagnoseDeepseek") && contents.contains("apiKey"),
        "bindings should include apiKey parameter for diagnoseDeepseek"
    );
    assert!(
        contents.contains("getApiKey") && contents.contains("get_api_key"),
        "bindings should include getApiKey command"
    );
}
