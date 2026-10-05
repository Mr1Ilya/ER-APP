import os
import shutil
import zipfile
import subprocess
import json
from datetime import datetime, timezone

v = "1.1.0-b.1"
base_dir = r"C:\dev\end-rage\ER-Launcher\target\release\bundle\nsis"
src_setup = os.path.join(base_dir, f"EndRage APP_{v}_x64-setup.exe")

dst_launcher_space = os.path.join(base_dir, f"EndRage Launcher_{v}_x64-setup.exe")
dst_launcher_dot = os.path.join(base_dir, f"EndRage.Launcher_{v}_x64-setup.exe")
dst_zip = os.path.join(base_dir, f"EndRage.Launcher_{v}_x64-setup.nsis.zip")

print("Copying setup executables...")
shutil.copy2(src_setup, dst_launcher_space)
shutil.copy2(src_setup, dst_launcher_dot)

print("Creating nsis.zip (ZIP_STORED)...")
with zipfile.ZipFile(dst_zip, "w", zipfile.ZIP_STORED) as zf:
    zf.write(dst_launcher_space, arcname=f"EndRage Launcher_{v}_x64-setup.exe")

print("Signing files with tauri signer...")
key_path = os.path.expanduser(r"~/.tauri/endrage.key")
password = "ERLauncher_9fK2#mX9$vQ7!zL4@EndRage"

files_to_sign = [src_setup, dst_launcher_space, dst_launcher_dot, dst_zip]

for f in files_to_sign:
    cmd = [
        "npx", "tauri", "signer", "sign",
        "-f", key_path,
        "-p", password,
        f
    ]
    print(f"Signing {os.path.basename(f)}...")
    res = subprocess.run(cmd, cwd=r"C:\dev\end-rage\ER-Launcher\apps\app", shell=True, capture_output=True, text=True)
    if res.returncode != 0:
        print("Sign error:", res.stderr)
    else:
        print("Signed OK")

print("Applying Windows Authenticode signature: Самозанятый Удалов Илья Игоревич - EndRage Team...")
ps_script = f"""
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$cert = Get-ChildItem Cert:\\CurrentUser\\My | Where-Object {{ $_.Subject -like "*Удалов Илья Игоревич*" }} | Select-Object -First 1
$files = @(
    "{src_setup}",
    "{dst_launcher_space}",
    "{dst_launcher_dot}"
)
foreach ($f in $files) {{
    if (Test-Path $f) {{
        Set-AuthenticodeSignature -FilePath $f -Certificate $cert -TimestampServer "http://timestamp.digicert.com"
    }}
}}
"""
ps_res = subprocess.run(["powershell", "-NoProfile", "-Command", ps_script], capture_output=True, text=True)
print(ps_res.stdout)

sig_file = dst_zip + ".sig"
with open(sig_file, "r", encoding="utf-8") as f:
    zip_sig = f.read().strip()

latest_data = {
    "version": v,
    "notes": f"EndRage APP v{v}: Переход на структуру версий 1.1.0-b.1, встроенная поддержка совместной игры (LAN / Radmin) без ошибки недействительной сессии, цифровая подпись издателя и подготовка кроссплатформенной поддержки Linux и macOS.",
    "pub_date": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "platforms": {
        "windows-x86_64": {
            "signature": zip_sig,
            "url": f"https://releases.end-rage.ru/EndRage.Launcher_{v}_x64-setup.nsis.zip"
        }
    }
}

latest_json_path = os.path.join(base_dir, "latest.json")
with open(latest_json_path, "w", encoding="utf-8") as f:
    json.dump(latest_data, f, ensure_ascii=False)

print(f"Created latest.json for v{v} successfully!")
