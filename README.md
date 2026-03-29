# ue-doctor

Health checker for UE5 projects. Scans your .uproject and project directory for common setup issues, misconfigurations, and packaging blockers.

## Install

```bash
cargo install --path .
# or use the binary directly:
./target/release/ue-doctor
```

## Usage

```bash
# Scan a project
ue-doctor /path/to/MyProject.uproject

# JSON output (for CI)
ue-doctor /path/to/MyProject.uproject --json

# Only show problems
ue-doctor /path/to/MyProject.uproject --problems-only
```

## Rules (25)

### Project File
- **P001** .uproject valid JSON
- **P002** Engine association set

### Modules
- **M001** Modules listed in .uproject
- **M002** Module Source/ directories exist

### Targets & Build
- **T001** .Target.cs files exist
- **T002** Editor target exists
- **T003** Game target exists
- **B001** .Build.cs files exist

### Directories
- **D001** Content/ exists
- **D002** Config/ exists
- **D003** DerivedDataCache not committed

### Config
- **C001** DefaultEngine.ini exists
- **C002** DefaultGame.ini exists
- **C003** Default map configured
- **C004** No stale config backups

### Plugins
- **PL001** Enabled plugins have .uplugin files
- **PL002** No duplicate plugin entries
- **PL003** Plugin directories have .uplugin

### Source Control
- **SC001** .gitignore exists (git repos)
- **SC002** Intermediate/Saved/DDC gitignored
- **SC003** .gitattributes exists
- **SC004** .uasset tracked by LFS

### Assets
- **A001** No spaces in project path
- **A002** Content/ naming conventions
- **A003** StarterContent removed
- **A004** No empty Content/ folders

## License

MIT
