use clap::Parser;
use colored::*;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "ue-doctor", about = "Health checker for UE5 projects")]
struct Cli {
    /// Path to the .uproject file
    project: PathBuf,

    /// Output as JSON instead of colored terminal
    #[arg(long)]
    json: bool,

    /// Only show warnings and errors (hide passing rules)
    #[arg(long)]
    problems_only: bool,
}

#[derive(Debug, Clone, Serialize)]
enum Severity {
    Pass,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize)]
struct Finding {
    rule_id: String,
    severity: Severity,
    message: String,
    detail: String,
}

#[derive(Debug, Serialize)]
struct Report {
    project: String,
    findings: Vec<Finding>,
    pass_count: usize,
    warn_count: usize,
    error_count: usize,
}

fn main() {
    let cli = Cli::parse();

    if !cli.project.exists() {
        eprintln!("Error: {} not found", cli.project.display());
        std::process::exit(1);
    }

    let project_dir = cli.project.parent().unwrap_or(Path::new("."));
    let uproject_content = fs::read_to_string(&cli.project).unwrap_or_default();

    let mut findings = Vec::new();

    check_uproject_valid(&uproject_content, &mut findings);
    check_engine_association(&uproject_content, &mut findings);
    check_modules_listed(&uproject_content, project_dir, &mut findings);
    check_target_files(project_dir, &mut findings);
    check_build_cs_files(project_dir, &mut findings);
    check_content_dir_exists(project_dir, &mut findings);
    check_config_dir(project_dir, &mut findings);
    check_default_engine_ini(project_dir, &mut findings);
    check_default_game_ini(project_dir, &mut findings);
    check_plugins_valid(&uproject_content, project_dir, &mut findings);
    check_duplicate_plugins(&uproject_content, &mut findings);
    check_gitignore(project_dir, &mut findings);
    check_git_lfs(project_dir, &mut findings);
    check_spaces_in_path(project_dir, &mut findings);
    check_content_naming(project_dir, &mut findings);
    check_default_map_set(project_dir, &mut findings);
    check_no_starter_content(project_dir, &mut findings);
    check_binary_config_files(project_dir, &mut findings);
    check_empty_folders(project_dir, &mut findings);
    check_plugin_uplugin_files(project_dir, &mut findings);
    check_derived_data_cache(project_dir, &mut findings);

    let pass_count = findings.iter().filter(|f| matches!(f.severity, Severity::Pass)).count();
    let warn_count = findings.iter().filter(|f| matches!(f.severity, Severity::Warning)).count();
    let error_count = findings.iter().filter(|f| matches!(f.severity, Severity::Error)).count();

    let report = Report {
        project: cli.project.display().to_string(),
        findings: findings.clone(),
        pass_count,
        warn_count,
        error_count,
    };

    if cli.json {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else {
        println!("{}", "UE Doctor Report".bold());
        println!("Project: {}\n", cli.project.display());

        for f in &findings {
            if cli.problems_only && matches!(f.severity, Severity::Pass) {
                continue;
            }
            let icon = match f.severity {
                Severity::Pass => "PASS".green(),
                Severity::Warning => "WARN".yellow(),
                Severity::Error => "FAIL".red(),
            };
            println!("[{}] {} {}", icon, f.rule_id.dimmed(), f.message);
            if !f.detail.is_empty() && !matches!(f.severity, Severity::Pass) {
                println!("       {}", f.detail.dimmed());
            }
        }

        println!();
        println!(
            "{} pass, {} warnings, {} errors",
            pass_count.to_string().green(),
            warn_count.to_string().yellow(),
            error_count.to_string().red()
        );
    }

    if error_count > 0 {
        std::process::exit(1);
    }
}

fn finding(id: &str, sev: Severity, msg: &str, detail: &str) -> Finding {
    Finding {
        rule_id: id.into(),
        severity: sev,
        message: msg.into(),
        detail: detail.into(),
    }
}

fn check_uproject_valid(content: &str, f: &mut Vec<Finding>) {
    match serde_json::from_str::<serde_json::Value>(content) {
        Ok(_) => f.push(finding("P001", Severity::Pass, ".uproject is valid JSON", "")),
        Err(e) => f.push(finding("P001", Severity::Error, ".uproject is not valid JSON", &e.to_string())),
    }
}

fn check_engine_association(content: &str, f: &mut Vec<Finding>) {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(content) {
        if v.get("EngineAssociation").and_then(|e| e.as_str()).is_some() {
            f.push(finding("P002", Severity::Pass, "Engine association is set", ""));
        } else {
            f.push(finding("P002", Severity::Warning, "No EngineAssociation in .uproject",
                "Project may not open with the correct engine version"));
        }
    }
}

fn check_modules_listed(content: &str, dir: &Path, f: &mut Vec<Finding>) {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(content) else { return };
    let Some(modules) = v.get("Modules").and_then(|m| m.as_array()) else { return };
    if modules.is_empty() {
        f.push(finding("M001", Severity::Warning, "No modules listed in .uproject", ""));
        return;
    }
    for m in modules {
        if let Some(name) = m.get("Name").and_then(|n| n.as_str()) {
            if !dir.join("Source").join(name).exists() {
                f.push(finding("M002", Severity::Error,
                    &format!("Module '{}' listed but Source/{} missing", name, name), ""));
            }
        }
    }
    f.push(finding("M001", Severity::Pass, &format!("{} module(s) listed", modules.len()), ""));
}

fn check_target_files(dir: &Path, f: &mut Vec<Finding>) {
    let source = dir.join("Source");
    if !source.exists() { return; }
    let targets: Vec<_> = glob::glob(&format!("{}/*.Target.cs", source.display()))
        .unwrap().flatten().collect();
    if targets.is_empty() {
        f.push(finding("T001", Severity::Error, "No .Target.cs files in Source/",
            "Need at least GameTarget and EditorTarget"));
        return;
    }
    let has_editor = targets.iter().any(|t| t.display().to_string().contains("Editor.Target"));
    let has_game = targets.iter().any(|t| {
        let s = t.display().to_string();
        !s.contains("Editor.Target") && !s.contains("Server.Target")
    });
    if !has_editor {
        f.push(finding("T002", Severity::Warning, "No Editor .Target.cs found", ""));
    }
    if !has_game {
        f.push(finding("T003", Severity::Warning, "No Game .Target.cs found", ""));
    }
    if has_editor && has_game {
        f.push(finding("T001", Severity::Pass, &format!("{} target(s)", targets.len()), ""));
    }
}

fn check_build_cs_files(dir: &Path, f: &mut Vec<Finding>) {
    let source = dir.join("Source");
    if !source.exists() { return; }
    let pattern = format!("{}/**/*.Build.cs", source.display());
    let builds: Vec<_> = glob::glob(&pattern).unwrap().flatten().collect();
    if builds.is_empty() {
        f.push(finding("B001", Severity::Error, "No .Build.cs files found",
            "Each module needs a Build.cs"));
    } else {
        f.push(finding("B001", Severity::Pass, &format!("{} Build.cs file(s)", builds.len()), ""));
    }
}

fn check_content_dir_exists(dir: &Path, f: &mut Vec<Finding>) {
    if dir.join("Content").exists() {
        f.push(finding("D001", Severity::Pass, "Content/ exists", ""));
    } else {
        f.push(finding("D001", Severity::Error, "Content/ missing", "Every UE project needs Content/"));
    }
}

fn check_config_dir(dir: &Path, f: &mut Vec<Finding>) {
    if dir.join("Config").exists() {
        f.push(finding("D002", Severity::Pass, "Config/ exists", ""));
    } else {
        f.push(finding("D002", Severity::Error, "Config/ missing", "Project config files required"));
    }
}

fn check_default_engine_ini(dir: &Path, f: &mut Vec<Finding>) {
    if dir.join("Config/DefaultEngine.ini").exists() {
        f.push(finding("C001", Severity::Pass, "DefaultEngine.ini exists", ""));
    } else {
        f.push(finding("C001", Severity::Warning, "DefaultEngine.ini missing", ""));
    }
}

fn check_default_game_ini(dir: &Path, f: &mut Vec<Finding>) {
    if dir.join("Config/DefaultGame.ini").exists() {
        f.push(finding("C002", Severity::Pass, "DefaultGame.ini exists", ""));
    } else {
        f.push(finding("C002", Severity::Warning, "DefaultGame.ini missing",
            "Needed for packaging"));
    }
}

fn check_plugins_valid(content: &str, dir: &Path, f: &mut Vec<Finding>) {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(content) else { return };
    let Some(plugins) = v.get("Plugins").and_then(|p| p.as_array()) else { return };
    for p in plugins {
        let name = p.get("Name").and_then(|n| n.as_str()).unwrap_or("?");
        let enabled = p.get("Enabled").and_then(|e| e.as_bool()).unwrap_or(false);
        if enabled {
            let local = dir.join("Plugins").join(name);
            let uplugin = local.join(format!("{}.uplugin", name));
            if local.exists() && !uplugin.exists() {
                f.push(finding("PL001", Severity::Error,
                    &format!("Plugin '{}' dir exists but .uplugin missing", name),
                    &format!("Expected: {}", uplugin.display())));
            }
        }
    }
}

fn check_duplicate_plugins(content: &str, f: &mut Vec<Finding>) {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(content) else { return };
    let Some(plugins) = v.get("Plugins").and_then(|p| p.as_array()) else { return };
    let mut seen = std::collections::HashSet::new();
    for p in plugins {
        let name = p.get("Name").and_then(|n| n.as_str()).unwrap_or("?");
        if !seen.insert(name.to_string()) {
            f.push(finding("PL002", Severity::Warning,
                &format!("Duplicate plugin entry: '{}'", name), ""));
        }
    }
}

fn check_gitignore(dir: &Path, f: &mut Vec<Finding>) {
    if !dir.join(".git").exists() { return; }
    let gi_path = dir.join(".gitignore");
    if !gi_path.exists() {
        f.push(finding("SC001", Severity::Warning, ".gitignore missing",
            "Intermediate/, Saved/, DerivedDataCache/ should be ignored"));
        return;
    }
    let content = fs::read_to_string(&gi_path).unwrap_or_default();
    for d in ["Intermediate", "Saved", "DerivedDataCache"] {
        if !content.contains(d) {
            f.push(finding("SC002", Severity::Warning,
                &format!("'{}' not in .gitignore", d), "Should not be committed"));
        }
    }
}

fn check_git_lfs(dir: &Path, f: &mut Vec<Finding>) {
    if !dir.join(".git").exists() { return; }
    let ga = dir.join(".gitattributes");
    if !ga.exists() {
        f.push(finding("SC003", Severity::Warning, "No .gitattributes (no Git LFS)",
            "Binary assets should use Git LFS"));
    } else {
        let content = fs::read_to_string(&ga).unwrap_or_default();
        if !content.contains("uasset") {
            f.push(finding("SC004", Severity::Warning,
                ".gitattributes doesn't track .uasset with LFS",
                "Add: *.uasset filter=lfs diff=lfs merge=lfs -text"));
        }
    }
}

fn check_spaces_in_path(dir: &Path, f: &mut Vec<Finding>) {
    let s = dir.display().to_string();
    if s.contains(' ') {
        f.push(finding("A001", Severity::Warning, "Project path contains spaces",
            &format!("{} -- can cause build issues", s)));
    } else {
        f.push(finding("A001", Severity::Pass, "No spaces in project path", ""));
    }
}

fn check_content_naming(dir: &Path, f: &mut Vec<Finding>) {
    let content = dir.join("Content");
    if !content.exists() { return; }
    let bad: Vec<String> = fs::read_dir(&content).into_iter().flatten().flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            if name.contains(' ') || name.starts_with('.') { Some(name) } else { None }
        }).collect();
    if bad.is_empty() {
        f.push(finding("A002", Severity::Pass, "Content/ naming is clean", ""));
    } else {
        f.push(finding("A002", Severity::Warning,
            &format!("{} content folders with bad names", bad.len()),
            &bad.join(", ")));
    }
}

fn check_default_map_set(dir: &Path, f: &mut Vec<Finding>) {
    let ini = dir.join("Config/DefaultEngine.ini");
    if !ini.exists() { return; }
    let content = fs::read_to_string(&ini).unwrap_or_default();
    if content.contains("GameDefaultMap") || content.contains("EditorStartupMap") {
        f.push(finding("C003", Severity::Pass, "Default map configured", ""));
    } else {
        f.push(finding("C003", Severity::Warning, "No default map set",
            "Set GameDefaultMap in DefaultEngine.ini"));
    }
}

fn check_no_starter_content(dir: &Path, f: &mut Vec<Finding>) {
    if dir.join("Content/StarterContent").exists() {
        f.push(finding("A003", Severity::Warning, "StarterContent still present",
            "Remove before packaging to reduce size"));
    }
}

fn check_binary_config_files(dir: &Path, f: &mut Vec<Finding>) {
    let config = dir.join("Config");
    if !config.exists() { return; }
    for entry in fs::read_dir(&config).into_iter().flatten().flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(".ini.bak") || name.ends_with(".ini.old") {
            f.push(finding("C004", Severity::Warning,
                &format!("Stale config backup: Config/{}", name), "Remove .bak/.old files"));
        }
    }
}

fn check_empty_folders(dir: &Path, f: &mut Vec<Finding>) {
    let content = dir.join("Content");
    if !content.exists() { return; }
    let empty: Vec<String> = fs::read_dir(&content).into_iter().flatten().flatten()
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter(|e| fs::read_dir(e.path()).map(|mut d| d.next().is_none()).unwrap_or(false))
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    if !empty.is_empty() {
        f.push(finding("A004", Severity::Warning,
            &format!("{} empty folder(s) in Content/", empty.len()),
            &empty.join(", ")));
    }
}

fn check_plugin_uplugin_files(dir: &Path, f: &mut Vec<Finding>) {
    let plugins = dir.join("Plugins");
    if !plugins.exists() { return; }
    for entry in fs::read_dir(&plugins).into_iter().flatten().flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) { continue; }
        let name = entry.file_name().to_string_lossy().to_string();
        let has_uplugin = fs::read_dir(entry.path()).into_iter().flatten().flatten()
            .any(|e| e.file_name().to_string_lossy().ends_with(".uplugin"));
        if !has_uplugin {
            f.push(finding("PL003", Severity::Error,
                &format!("Plugin '{}' has no .uplugin", name),
                &format!("Expected: Plugins/{}/{}.uplugin", name, name)));
        }
    }
}

fn check_derived_data_cache(dir: &Path, f: &mut Vec<Finding>) {
    if !dir.join("DerivedDataCache").exists() { return; }
    if !dir.join(".git").exists() { return; }
    let gi = fs::read_to_string(dir.join(".gitignore")).unwrap_or_default();
    if !gi.contains("DerivedDataCache") {
        f.push(finding("D003", Severity::Warning,
            "DerivedDataCache/ exists and may not be gitignored",
            "Build cache should not be committed"));
    }
}
