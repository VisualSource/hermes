use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let out: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../api/openapi.yaml");

    let yaml = hermes_server::openapi().to_yaml()?;

    let file_name = out
        .file_name()
        .ok_or("output path has no file name")?
        .to_owned();
    let mut tmp = out.clone();
    tmp.set_file_name(format!("{}.tmp", file_name.to_string_lossy()));

    fs::write(&tmp, yaml)?;
    fs::rename(&tmp, &out)?;

    println!("wrote {}", out.display());
    Ok(())
}
