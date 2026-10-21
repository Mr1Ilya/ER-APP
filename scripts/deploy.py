import sys
import os
import json
import zipfile
import subprocess
import datetime
import urllib.request
import urllib.error

def get_gh_token():
    token = os.environ.get("GH_TOKEN")
    if token:
        return token
    token_file = os.path.expanduser("~/.tauri/gh_token")
    if os.path.exists(token_file):
        with open(token_file, "r", encoding="utf-8") as f:
            return f.read().strip()
    return ""

GH_TOKEN = get_gh_token()
REPO = "Mr1Ilya/ER-APP"
REMOTE_HOST = "webapps@2.26.49.76"
REMOTE_DIR = "/home/webapps/apps/launcher-backend/storage/releases"
KEY_PATH = os.path.expanduser("~/.tauri/endrage.key")
KEY_PASS = "ERLauncher_9fK2#mX9$vQ7!zL4@EndRage"

def log(msg):
    print(msg, flush=True)

def get_current_version():
    tauri_conf = os.path.join("apps", "app", "src-tauri", "tauri.conf.json")
    if os.path.exists(tauri_conf):
        with open(tauri_conf, "r", encoding="utf-8") as f:
            data = json.load(f)
            v = data.get("package", {}).get("version")
            if v:
                return v
    pkg_json = os.path.join("apps", "app-frontend", "package.json")
    with open(pkg_json, "r", encoding="utf-8") as f:
        data = json.load(f)
        return data.get("version")

def main():
    skip_build = "--skip-build" in sys.argv or "--no-build" in sys.argv
    version = None
    for arg in sys.argv[1:]:
        if not arg.startswith("--"):
            version = arg
            break
    if not version:
        version = get_current_version()
    
    log(f"Deploying EndRage Launcher v{version}...")

    if not skip_build:
        log("Compiling app:build...")
        res = subprocess.run(["pnpm", "run", "app:build"], shell=True)
        if res.returncode != 0:
            log("Build failed!")
            sys.exit(1)

    bundle_dir = os.path.abspath(os.path.join("target", "release", "bundle", "nsis"))
    if not os.path.isdir(bundle_dir):
        log(f"Directory not found: {bundle_dir}")
        sys.exit(1)

    candidates = [
        f"EndRage Launcher_{version}_x64-setup.exe",
        f"EndRage.Launcher_{version}_x64-setup.exe",
        f"EndRage APP_{version}_x64-setup.exe",
        f"EndRage.APP_{version}_x64-setup.exe"
    ]

    source_exe = None
    for name in candidates:
        p = os.path.join(bundle_dir, name)
        if os.path.exists(p):
            source_exe = p
            break

    if not source_exe:
        log(f"Error: No setup executable found in {bundle_dir} for v{version}")
        sys.exit(1)

    clean_exe_name = f"EndRage.Launcher_{version}_x64-setup.exe"
    clean_exe_path = os.path.join(bundle_dir, clean_exe_name)
    if clean_exe_path != source_exe:
        with open(source_exe, "rb") as src, open(clean_exe_path, "wb") as dst:
            dst.write(src.read())

    zip_name = f"EndRage.Launcher_{version}_x64-setup.nsis.zip"
    zip_path = os.path.join(bundle_dir, zip_name)
    exe_in_zip = f"EndRage Launcher_{version}_x64-setup.exe"

    log(f"Packing {zip_name} strictly uncompressed (ZIP_STORED, method 0)...")
    with zipfile.ZipFile(zip_path, "w", compression=zipfile.ZIP_STORED) as zf:
        zf.write(source_exe, arcname=exe_in_zip)

    log("Signing with Minisign...")
    sign_cmd = [
        "pnpm", "--filter", "@erteam/app", "tauri", "signer", "sign",
        "-f", KEY_PATH,
        "-p", KEY_PASS,
        zip_path
    ]
    res = subprocess.run(sign_cmd, shell=True, capture_output=True, text=True)
    sig_path = zip_path + ".sig"
    if not os.path.exists(sig_path):
        log(f"Signing failed:\n{res.stdout}\n{res.stderr}")
        sys.exit(1)

    with open(sig_path, "r", encoding="utf-8") as f:
        signature = f.read().strip()

    exe_sig_cmd = [
        "pnpm", "--filter", "@erteam/app", "tauri", "signer", "sign",
        "-f", KEY_PATH,
        "-p", KEY_PASS,
        clean_exe_path
    ]
    subprocess.run(exe_sig_cmd, shell=True, capture_output=True, text=True)
    exe_sig_path = clean_exe_path + ".sig"

    pub_date = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    notes = f"Release v{version}"

    latest_data = {
        "version": version,
        "notes": notes,
        "pub_date": pub_date,
        "platforms": {
            "windows-x86_64": {
                "signature": signature,
                "url": f"https://github.com/{REPO}/releases/download/v{version}/{zip_name}"
            }
        }
    }

    latest_json_path = os.path.join(bundle_dir, "latest.json")
    with open(latest_json_path, "w", encoding="utf-8") as f:
        json.dump(latest_data, f, indent=2, ensure_ascii=False)

    log("Uploading to releases.end-rage.ru via SCP...")
    upload_files = [zip_path, sig_path, latest_json_path, clean_exe_path]
    if os.path.exists(exe_sig_path):
        upload_files.append(exe_sig_path)

    scp_cmd = ["scp", "-o", "BatchMode=yes"] + upload_files + [f"{REMOTE_HOST}:{REMOTE_DIR}/"]
    res = subprocess.run(scp_cmd, capture_output=True, text=True)
    if res.returncode != 0:
        log(f"SCP error: {res.stderr}")
    else:
        log("SCP upload successful!")

    log("Creating server aliases...")
    alias_cmd = (
        f"cd {REMOTE_DIR} && "
        f"cp -f '{clean_exe_name}' 'EndRage Launcher_{version}_x64-setup.exe' && "
        f"cp -f '{clean_exe_name}' 'EndRage APP_{version}_x64-setup.exe' && "
        f"cp -f '{clean_exe_name}' 'EndRage.APP_{version}_x64-setup.exe'"
    )
    subprocess.run(["ssh", "-o", "BatchMode=yes", REMOTE_HOST, alias_cmd], capture_output=True)

    log("Syncing with GitHub Releases...")
    headers = {
        "Authorization": f"Bearer {GH_TOKEN}",
        "Accept": "application/vnd.github+json",
        "User-Agent": "EndRage-Deployer"
    }

    release_info = None
    try:
        req = urllib.request.Request(f"https://api.github.com/repos/{REPO}/releases/tags/v{version}", headers=headers)
        with urllib.request.urlopen(req) as resp:
            release_info = json.loads(resp.read().decode("utf-8"))
    except Exception as e:
        log(f"Could not fetch release tag v{version}: {e}")

    if release_info:
        release_id = release_info["id"]
        existing_assets = {a["name"]: a["id"] for a in release_info.get("assets", [])}
        gh_assets = {
            zip_name: zip_path,
            f"{zip_name}.sig": sig_path,
            "latest.json": latest_json_path,
            clean_exe_name: clean_exe_path
        }

        for fname, fpath in gh_assets.items():
            if not os.path.exists(fpath):
                continue
            if fname in existing_assets:
                del_url = f"https://api.github.com/repos/{REPO}/releases/assets/{existing_assets[fname]}"
                del_req = urllib.request.Request(del_url, headers=headers, method="DELETE")
                try:
                    with urllib.request.urlopen(del_req):
                        pass
                except Exception:
                    pass

            with open(fpath, "rb") as f:
                fdata = f.read()
            up_url = f"https://uploads.github.com/repos/{REPO}/releases/{release_id}/assets?name={fname}"
            up_headers = dict(headers)
            up_headers["Content-Type"] = "application/octet-stream"
            up_req = urllib.request.Request(up_url, data=fdata, headers=up_headers, method="POST")
            try:
                with urllib.request.urlopen(up_req):
                    log(f"GitHub release asset synced: {fname}")
            except Exception as e:
                log(f"GitHub asset upload failed for {fname}: {e}")

    log("Deployment completed successfully!")

if __name__ == "__main__":
    main()
