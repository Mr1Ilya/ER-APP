import os
import sys
import json
import requests

TOKEN = os.environ.get("GITHUB_TOKEN")
if not TOKEN:
    token_file = os.path.expanduser("~/.github_token")
    if os.path.exists(token_file):
        with open(token_file, "r", encoding="utf-8") as f:
            TOKEN = f.read().strip()

if not TOKEN:
    print("GITHUB_TOKEN not found in env or ~/.github_token")
    sys.exit(1)

REPO = "Mr1Ilya/ER-APP"
HEADERS = {
    "Authorization": f"Bearer {TOKEN}",
    "Accept": "application/vnd.github.v3+json",
    "User-Agent": "python-release-uploader"
}

def publish_release(version, tag_name, name, body, file_paths):
    print(f"Creating GitHub Release for {tag_name}...")
    release_payload = {
        "tag_name": tag_name,
        "name": name,
        "body": body,
        "draft": False,
        "prerelease": False
    }
    
    url = f"https://api.github.com/repos/{REPO}/releases"
    resp = requests.post(url, headers=HEADERS, json=release_payload)
    
    if resp.status_code == 422: # Release or tag might exist
        print("Release might already exist, fetching existing release...")
        get_resp = requests.get(f"https://api.github.com/repos/{REPO}/releases/tags/{tag_name}", headers=HEADERS)
        if get_resp.status_code == 200:
            release_data = get_resp.json()
        else:
            print("Failed to get or create release:", resp.text)
            return
    elif resp.status_code in (200, 201):
        release_data = resp.json()
    else:
        print(f"Failed to create release ({resp.status_code}):", resp.text)
        return

    release_id = release_data["id"]
    upload_url_template = release_data["upload_url"] # e.g. https://uploads.github.com/.../assets{?name,label}
    upload_base = upload_url_template.split("{")[0]
    
    print(f"Release {tag_name} created (ID: {release_id}). Uploading assets...")
    
    existing_assets = {a["name"]: a["id"] for a in release_data.get("assets", [])}

    for path in file_paths:
        if not os.path.exists(path):
            print(f"File not found: {path}, skipping.")
            continue
        
        file_name = os.path.basename(path)
        
        # If asset already uploaded, delete it first
        if file_name in existing_assets:
            print(f"Asset {file_name} already exists, deleting first...")
            del_url = f"https://api.github.com/repos/{REPO}/releases/assets/{existing_assets[file_name]}"
            requests.delete(del_url, headers=HEADERS)
        
        print(f"Uploading {file_name} ({os.path.getsize(path)} bytes)...")
        upload_url = f"{upload_base}?name={file_name}"
        headers = {
            "Authorization": f"Bearer {TOKEN}",
            "Content-Type": "application/octet-stream",
            "User-Agent": "python-release-uploader"
        }
        
        with open(path, "rb") as f:
            up_resp = requests.post(upload_url, headers=headers, data=f)
            if up_resp.status_code in (200, 201):
                print(f"  [OK] {file_name} uploaded successfully!")
            else:
                print(f"  [ERR] Failed to upload {file_name}: {up_resp.status_code} {up_resp.text}")

if __name__ == "__main__":
    v = "1.0.21"
    base_dir = r"C:\dev\end-rage\ER-Launcher\target\release\bundle\nsis"
    files = [
        os.path.join(base_dir, f"EndRage.Launcher_{v}_x64-setup.nsis.zip"),
        os.path.join(base_dir, f"EndRage.Launcher_{v}_x64-setup.nsis.zip.sig"),
        os.path.join(base_dir, f"EndRage APP_{v}_x64-setup.exe"),
        os.path.join(base_dir, f"EndRage APP_{v}_x64-setup.exe.sig"),
        os.path.join(base_dir, f"EndRage Launcher_{v}_x64-setup.exe"),
        os.path.join(base_dir, "latest.json"),
    ]
    notes = (
        f"### EndRage APP v{v}\n\n"
        "- Исправлен запуск и скачивание сборок при установке на другой диск (диск D / произвольный путь)\n"
        "- Автоматическое восстановление и докачивание отсутствующих client.jar и JRE Java при запуске\n"
        "- Устранена блокировка 'Instance is already running as process' при сбоях или зависаниях процессов\n"
        "- Защита от потери путей кастомной директории в настройках и ложных ошибок ранней инициализации"
    )
    publish_release(v, f"v{v}", f"EndRage Launcher v{v}", notes, files)
