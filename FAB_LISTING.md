# Fab Listing: UE Doctor

## Title
UE Doctor - Project Health Checker for UE5

## Tagline
25 rules. Catches config issues, plugin conflicts, source control problems, and packaging blockers. Free and open source.

## Category
Tools & Plugins > Engine Tools

## Price
Free

## Tags
Project Health, Config, Validation, Source Control, Packaging, CI, Automation, Editor Tools

## Description

UE Doctor scans your .uproject and project directory for common setup issues, misconfigurations, and packaging blockers. Runs 25 rules across 7 categories.

**Categories:**

Project File (2 rules): Valid .uproject JSON, engine association set.

Modules (2 rules): Modules listed in .uproject, Source directories exist.

Targets and Build (3 rules): .Target.cs files exist, Editor and Game targets present, .Build.cs files exist.

Directories (3 rules): Content/ exists, Config/ exists, DerivedDataCache not committed.

Config (4 rules): DefaultEngine.ini and DefaultGame.ini exist, default map configured, no stale config backups.

Plugins (3 rules): Enabled plugins have .uplugin files, no duplicate entries, plugin directories have .uplugin.

Source Control (4 rules): .gitignore exists, Intermediate/Saved/DDC gitignored, .gitattributes exists, .uasset tracked by LFS.

Assets (4 rules): No spaces in project path, Content/ naming conventions, StarterContent removed, no empty Content/ folders.

**Usage:**
```
ue-doctor /path/to/MyProject.uproject
ue-doctor /path/to/MyProject.uproject --json
ue-doctor /path/to/MyProject.uproject --problems-only
```

**Requirements:**
- Rust (cargo install) or prebuilt binary
- A UE5 project directory

**Also from Silent Factory:**
- Unreal MCP ($19.99): 140 tools for full Blueprint read/write, level editing, materials, animation, and more.

## Technical Details

Features:
- 25 project health rules across 7 categories
- JSON output for CI integration
- Problems-only mode for quick checks
- Rust CLI (fast, single binary)

Number of Blueprints: 0
Number of C++ Classes: 0
Network Replicated: No
Supported Development Platforms: Windows
Documentation: https://github.com/tencircles/ue-doctor
