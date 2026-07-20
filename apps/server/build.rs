use oxc::{
    allocator::Allocator,
    codegen::{Codegen, CodegenOptions},
    diagnostics::Diagnostics,
    minifier::{Minifier, MinifierOptions},
    parser::Parser,
    semantic::SemanticBuilder,
    span::SourceType,
    transformer::{EnvOptions, TransformOptions, Transformer, TypeScriptOptions},
};
use std::{env, fs, path::PathBuf, process::Command};

const BROWSER_LIST_QUERY: &'static str = "last 2 versions";

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("failed to get manifest dir");
    let project_dir = PathBuf::from(manifest_dir);

    build_proto(&project_dir);
    build_ui(&project_dir);
}

fn build_proto(project_dir: &PathBuf) {
    let protoc_source_dir = project_dir.join("../../api/proto/");

    let files = std::fs::read_dir(&protoc_source_dir).expect("failed to read dir");

    let proto_files = files
        .into_iter()
        .map(|file| {
            let path = file.expect("failed to get dir").path();

            path
        })
        .collect::<Vec<PathBuf>>();

    println!("Found {} proto files", proto_files.len());

    prost_build::compile_protos(&proto_files, &[protoc_source_dir])
        .expect("failed to compile proto");

    build_ui(&project_dir);
}

fn build_ui(project_dir: &PathBuf) {
    #[cfg(target_os = "windows")]
    let pnpm_cmd = "pnpm.cmd";

    #[cfg(not(target_os = "windows"))]
    let pnpm_cmd = "pnpm";

    let source_type = SourceType::ts().with_module(true);

    let allocator = Allocator::default();
    let source_dir = project_dir.join("./src/ui");
    let sources = fs::read_dir(source_dir).expect("failed to read source directory");

    let transform_opts = TransformOptions {
        env: EnvOptions::from_browserslist_query(BROWSER_LIST_QUERY)
            .expect("failed to parse browser list query"),
        typescript: TypeScriptOptions::default(),
        ..Default::default()
    };

    for source_result in sources {
        let source = source_result.expect("failed to get dir entry");
        let file_type = source.file_type().expect("failed to get file type");
        if !file_type.is_file() {
            continue;
        }
        let file_path = source.path();
        let file_name = file_path.file_stem().expect("failed to get file stem");

        let name = file_path.file_name().expect("failed to get name");

        let ext = file_path.extension().expect("failed to get file ext");

        match ext.to_str().expect("failed to get ext") {
            "ts" => {
                // parse
                let content = fs::read_to_string(&file_path).expect("failed to read source file");
                let ret = Parser::new(&allocator, &content, source_type).parse();
                if assert_diagnostics(&ret.diagnostics) {
                    return;
                }
                let mut program = ret.program;

                // semantic
                let semantic_ret = SemanticBuilder::new().build(&program);
                if assert_diagnostics(&semantic_ret.diagnostics) {
                    return;
                }
                let scoping = semantic_ret.semantic.into_scoping();

                // transform
                let transform_ret = Transformer::new(&allocator, &file_path, &transform_opts)
                    .build_with_scoping(scoping, &mut program);
                if assert_diagnostics(&transform_ret.diagnostics) {
                    return;
                }
                // minify
                let minify_ret =
                    Minifier::new(MinifierOptions::default()).minify(&allocator, &mut program);

                // codegen
                let codegen_opts = CodegenOptions {
                    minify: true,
                    ..Default::default()
                };

                let codegen_ret = Codegen::new()
                    .with_options(codegen_opts)
                    .with_scoping(minify_ret.scoping)
                    .build(&program);

                // write
                let mut output_file_path = project_dir.join("public/static").join(file_name);
                output_file_path.add_extension("js");

                fs::write(output_file_path, codegen_ret.code).expect("failed to write file");
            }
            "html" => {
                let outdir = project_dir.join("./public").join(name);
                fs::copy(file_path, outdir).expect("failed to copy html file");
            }
            "css" => {
                let outdir = project_dir.join("./public/static").join(name);

                Command::new(&pnpm_cmd)
                    .current_dir(project_dir)
                    .args([
                        "exec",
                        "tailwindcss",
                        "-i",
                        file_path.to_str().expect("failed to get path"),
                        "-o",
                        outdir.to_str().expect("failed to get path"),
                        "--minify",
                    ])
                    .spawn()
                    .expect("failed to run tailwind cli");
            }
            _ => {
                eprintln!("unsupported file founded in source dir {:?}", file_path)
            }
        }
    }

    let alpine_js_out = project_dir.join("./public/static/alpinejs.esm.min.js");
    if !alpine_js_out.exists() {
        let alpine_js_file = project_dir.join("./node_modules/alpinejs/dist/module.esm.min.js");
        fs::copy(alpine_js_file, alpine_js_out).expect("failed to copy alpinejs dep to output dir");
    }
}

fn assert_diagnostics(diag: &Diagnostics) -> bool {
    if !diag.is_empty() {
        println!("js pipe error: {:?}", diag);

        return true;
    }

    return false;
}
