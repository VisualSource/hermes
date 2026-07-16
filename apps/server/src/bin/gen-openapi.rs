use hermes_server::ApiDoc;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use utoipa::OpenApi;

fn main() -> Result<(), Box<dyn Error>> {
    let out: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../api/openapi.yaml");

    let yaml = ApiDoc::openapi().to_yaml()?;

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
