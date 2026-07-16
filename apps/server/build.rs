use std::{path::PathBuf, process::Command, str::FromStr};

fn main() {
    let protoc_dir = PathBuf::from_str("../../api/proto/").expect("failed to make protoc dir");

    let files = std::fs::read_dir("../../api/proto").expect("failed to read dir");

    let proto_files = files
        .into_iter()
        .map(|file| {
            let path = file.expect("failed to get dir").path();

            path
        })
        .collect::<Vec<PathBuf>>();

    println!("Found {} proto files", proto_files.len());

    prost_build::compile_protos(&proto_files, &[protoc_dir]).expect("failed to compile proto");

    build_ui();
}

fn build_ui() {
    Command::new("pnpm")
        .arg("tsc")
        .spawn()
        .expect("failed to run typescript");
}
