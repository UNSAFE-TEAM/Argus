use crate::cli::{Module, Preset};
use anyhow::Context;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct ScriptLoadOptions {
    pub preset: Option<Preset>,
    pub module: Option<Module>,
    pub scripts_dir: Option<PathBuf>,
}

struct EmbeddedScript {
    path: &'static str,
    content: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/embedded_scripts.rs"));

pub fn scripts(options: &ScriptLoadOptions) -> anyhow::Result<String> {
    let mut scripts = Vec::new();

    scripts.extend(load_runtime_scripts());

    if let Some(scripts_dir) = &options.scripts_dir {
        scripts.extend(load_filesystem_scripts(scripts_dir, options)?);
    } else {
        scripts.extend(load_embedded_scripts(options));
    }

    Ok(scripts.join("\n\n"))
}

fn load_runtime_scripts() -> Vec<String> {
    vec![
        include_str!("../../runtime/bootstrap/agent.v1.js").to_string(),
        include_str!("../../runtime/sensors/sensors.v1.js").to_string(),
    ]
}

fn load_embedded_scripts(options: &ScriptLoadOptions) -> Vec<String> {
    let mut scripts = Vec::new();

    scripts.extend(embedded_group("scripts/sensors/"));
    scripts.extend(embedded_group("scripts/anti_injection/"));
    scripts.extend(embedded_group("scripts/anti_debug/"));
    scripts.extend(embedded_group("scripts/anti_sandbox/"));
    scripts.extend(embedded_group("scripts/modules/behavior/"));

    if let Some(preset) = &options.preset {
        scripts.extend(embedded_group(&format!(
            "scripts/presets/{}/",
            preset.dir_name()
        )));
    }

    if let Some(module) = &options.module {
        scripts.extend(embedded_group(&format!(
            "scripts/modules/{}/",
            module.dir_name()
        )));
    }

    scripts
}

fn embedded_group(prefix: &str) -> Vec<String> {
    EMBEDDED_SCRIPTS
        .iter()
        .filter(|script| script.path.starts_with(prefix))
        .map(|script| script.content.to_string())
        .collect()
}

fn load_filesystem_scripts(
    scripts_dir: &Path,
    options: &ScriptLoadOptions,
) -> anyhow::Result<Vec<String>> {
    let mut paths = Vec::new();

    paths.extend(collect_js_scripts(&scripts_dir.join("sensors"))?);
    paths.extend(collect_js_scripts(&scripts_dir.join("anti_injection"))?);
    paths.extend(collect_js_scripts(&scripts_dir.join("anti_debug"))?);
    paths.extend(collect_js_scripts(&scripts_dir.join("anti_sandbox"))?);
    paths.extend(collect_js_scripts(
        &scripts_dir.join("modules").join("behavior"),
    )?);
    if let Some(preset) = &options.preset {
        paths.extend(collect_js_scripts(
            &scripts_dir.join("modules").join(preset.dir_name()),
        )?);
    }
    if let Some(module) = &options.module {
        paths.extend(collect_js_scripts(
            &scripts_dir.join("modules").join(module.dir_name()),
        )?);
    }
    read_scripts(paths)
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
