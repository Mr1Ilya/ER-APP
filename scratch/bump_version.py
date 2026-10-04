import json

# apps/app-frontend/package.json
pkg_path = r"apps/app-frontend/package.json"
with open(pkg_path, "r", encoding="utf-8") as f:
    pkg = json.load(f)
pkg["version"] = "1.0.21"
with open(pkg_path, "w", encoding="utf-8") as f:
    json.dump(pkg, f, indent="\t")
    f.write("\n")

# apps/app/Cargo.toml
app_cargo_path = r"apps/app/Cargo.toml"
with open(app_cargo_path, "r", encoding="utf-8") as f:
    content = f.read()
content = content.replace('version = "1.0.20"', 'version = "1.0.21"')
with open(app_cargo_path, "w", encoding="utf-8") as f:
    f.write(content)

# packages/app-lib/Cargo.toml
lib_cargo_path = r"packages/app-lib/Cargo.toml"
with open(lib_cargo_path, "r", encoding="utf-8") as f:
    content = f.read()
content = content.replace('version = "1.0.20"', 'version = "1.0.21"')
with open(lib_cargo_path, "w", encoding="utf-8") as f:
    f.write(content)

# apps/app/tauri.conf.json
tauri_conf_path = r"apps/app/tauri.conf.json"
with open(tauri_conf_path, "r", encoding="utf-8") as f:
    content = f.read()
content = content.replace('"version": "1.0.20"', '"version": "1.0.21"')
with open(tauri_conf_path, "w", encoding="utf-8") as f:
    f.write(content)

print("Bumped to 1.0.21 successfully!")
