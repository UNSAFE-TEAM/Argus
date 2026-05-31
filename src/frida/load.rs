use crate::cli::Preset;
use anyhow::Context;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct ScriptLoadOptions {
    pub preset: Option<Preset>,
}

pub fn scripts(options: &ScriptLoadOptions) -> anyhow::Result<String> {
    let mut scripts = Vec::new();

    scripts.extend(load_runtime_scripts());
    scripts.extend(load_filesystem_scripts(options)?);

    Ok(scripts.join("\n\n"))
}

fn load_runtime_scripts() -> Vec<String> {
    vec![
        include_str!("../../runtime/bootstrap/agent.v1.js").to_string(),
        include_str!("../../runtime/sensors/sensors.v1.js").to_string(),
    ]
}

fn load_filesystem_scripts(options: &ScriptLoadOptions) -> anyhow::Result<Vec<String>> {
    let scripts_dir = PathBuf::from("scripts");
    let mut paths = Vec::new();

    paths.extend(collect_js_scripts(&scripts_dir.join("sensors"))?);
    paths.extend(collect_js_scripts(&scripts_dir.join("anti_injection"))?);
    paths.extend(collect_js_scripts(&scripts_dir.join("anti_debug"))?);
    paths.extend(collect_js_scripts(&scripts_dir.join("anti_sandbox"))?);
    paths.extend(load_preset_paths(&scripts_dir, options.preset.as_ref())?);

    read_scripts(paths)
}

fn load_preset_paths(scripts_dir: &Path, preset: Option<&Preset>) -> anyhow::Result<Vec<PathBuf>> {
    let Some(preset) = preset else {
        return Ok(Vec::new());
    };

    collect_js_scripts(&scripts_dir.join("presets").join(preset.dir_name()))
}

fn read_scripts(paths: Vec<PathBuf>) -> anyhow::Result<Vec<String>> {
    paths
        .into_iter()
        .map(|path| {
            fs::read_to_string(&path)
                .with_context(|| format!("read script failed: {}", path.display()))
        })
        .collect()
}
fn collect_js_scripts(dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let entries = fs::read_dir(dir)
        .with_context(|| format!("read scripts directory failed: {}", dir.display()))?;

    let mut scripts = Vec::new();
    for entry in entries {
        let path = entry
            .with_context(|| format!("read scripts directory entry failed: {}", dir.display()))?
            .path();

        if path.is_dir() {
            scripts.extend(collect_js_scripts(&path)?);
        } else if path.extension().is_some_and(|ext| ext == "js") {
            scripts.push(path);
        }
    }

    scripts.sort();
    Ok(scripts)
}
