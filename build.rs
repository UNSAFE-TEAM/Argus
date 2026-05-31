use std::{
    env, fs,
    io::{self},
    path::{Path, PathBuf},
};

fn main() {
    println!("cargo:rerun-if-changed=scripts");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let scripts_dir = manifest_dir.join("scripts");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let output_path = out_dir.join("embedded_scripts.rs");

    let mut scripts = Vec::new();
    collect_js_scripts(&scripts_dir, &mut scripts).unwrap();
    scripts.sort();

    let mut output = String::new();
    output.push_str("const EMBEDDED_SCRIPTS: &[EmbeddedScript] = &[\n");

    for path in scripts {
        let relative = path.strip_prefix(&manifest_dir).unwrap();
        let relative = normalize_path(relative);
        let absolute = normalize_path(&path);

        output.push_str("    EmbeddedScript {\n");
        output.push_str(&format!("        path: {:?},\n", relative));
        output.push_str(&format!("        content: include_str!({:?}),\n", absolute));
        output.push_str("    },\n");
    }

    output.push_str("];\n");

    fs::write(output_path, output).unwrap();
}

fn collect_js_scripts(dir: &Path, scripts: &mut Vec<PathBuf>) -> io::Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let path = entry?.path();

        if path.is_dir() {
            collect_js_scripts(&path, scripts)?;
        } else if path.extension().is_some_and(|ext| ext == "js") {
            scripts.push(path);
        }
    }

    Ok(())
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
