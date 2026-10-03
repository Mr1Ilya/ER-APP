import urllib.request
import json

api_key = "$2a$10$bL4bIL5pUWqfcO7KQtnMReakwtfHbNKh6v1uTpKlzhwoueEJQnPnm"
url = "https://api.curseforge.com/v1/mods/files"
payload = json.dumps({"fileIds": [4828114, 5000000]}).encode('utf-8')
req = urllib.request.Request(url, data=payload, headers={'x-api-key': api_key, 'Content-Type': 'application/json', 'Accept': 'application/json'})
with urllib.request.urlopen(req) as resp:
    data = json.loads(resp.read().decode('utf-8'))
    print("Files found:", len(data.get('data', [])))
    for f in data.get('data', []):
        print(f"File {f['id']}: {f['fileName']} -> {f.get('downloadUrl')}")
