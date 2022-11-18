use std::env::current_dir;
use std::fs::create_dir_all;

use template_app_name::contract::TemplateApp;
use template_app_name::msg::ConfigResponse;
use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    TemplateApp::export_schema(&out_dir);
    export_schema(&schema_for!(ConfigResponse), &out_dir);
}
