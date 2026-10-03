import os
import zipfile
import shutil
import subprocess
import json

base_dir = r"C:\dev\end-rage\ER-Launcher\target\release\bundle\nsis"
key_path = os.path.expanduser(r"~/.tauri/endrage.key")
password = r"ERLauncher_9fK2#mX9$vQ7!zL4@EndRage"
version = "1.0.20"

# Expected built exe
app_exe = os.path.join(base_dir, f"EndRage APP_{version}_x64-setup.exe")
launcher_exe = os.path.join(base_dir, f"EndRage Launcher_{version}_x64-setup.exe")
dotted_exe = os.path.join(base_dir, f"EndRage.Launcher_{version}_x64-setup.exe")

if os.path.exists(app_exe) and not os.path.exists(launcher_exe):
    shutil.copyfile(app_exe, launcher_exe)
if os.path.exists(launcher_exe) and not os.path.exists(app_exe):
    shutil.copyfile(launcher_exe, app_exe)
if os.path.exists(launcher_exe):
    shutil.copyfile(launcher_exe, dotted_exe)

src_for_zip = launcher_exe if os.path.exists(launcher_exe) else app_exe
zip_path = os.path.join(base_dir, f"EndRage.Launcher_{version}_x64-setup.nsis.zip")

print("Packaging zip...")
with zipfile.ZipFile(zip_path, 'w', compression=zipfile.ZIP_STORED) as z:
    z.write(src_for_zip, arcname=os.path.basename(launcher_exe))

print(f"Zip created: {zip_path} ({os.path.getsize(zip_path)} bytes)")

# Function to sign
def sign(file_to_sign):
    cmd = ['pnpm', '--filter', '@erteam/app', 'tauri', 'signer', 'sign', '-f', key_path, '-p', password, file_to_sign]
    proc = subprocess.run(cmd, cwd=r"C:\dev\end-rage\ER-Launcher", capture_output=True, text=True, shell=True)
    out = proc.stdout + proc.stderr
    print(f"Signing {file_to_sign}: exit {proc.returncode}")
    sig = None
    if "Public signature:" in out:
        parts = out.split("Public signature:")
        sig_part = parts[1].split("Make sure to")[0].strip()
        sig = sig_part
    return sig

zip_sig = sign(zip_path)
launcher_sig = sign(launcher_exe)
app_sig = sign(app_exe)
dotted_sig = sign(dotted_exe)

print("ZIP Signature:")
print(zip_sig)

if zip_sig:
    latest = {
        "version": version,
        "notes": f"EndRage APP v{version}: Фоновая установка сборок CurseForge без блокировки экрана, надежная загрузка модов с таймаутами и повторами.",
        "pub_date": "2026-10-03T22:50:00Z",
        "platforms": {
            "windows-x86_64": {
                "signature": zip_sig,
                "url": f"https://releases.end-rage.ru/EndRage.Launcher_{version}_x64-setup.nsis.zip"
            }
        }
    }
    latest_json_path = os.path.join(base_dir, "latest.json")
    with open(latest_json_path, "w", encoding="utf-8") as f:
        json.dump(latest, f, indent=2)
    print("latest.json created successfully!")
