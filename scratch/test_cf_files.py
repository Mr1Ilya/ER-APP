import urllib.request
import json

api_key = "$2a$10$bL4bIL5pUWqfcO7KQtnMReakwtfHbNKh6v1uTpKlzhwoueEJQnPnm"
mod_id = 876781 # Better MC BMC4
url = f"https://api.curseforge.com/v1/mods/{mod_id}/files?pageSize=5"
req = urllib.request.Request(url, headers={'x-api-key': api_key, 'Accept': 'application/json'})
with urllib.request.urlopen(req) as resp:
    data = json.loads(resp.read().decode('utf-8'))
    print("Files count:", len(data.get('data', [])))
    for f in data.get('data', []):
        print(f"File ID: {f['id']}, DisplayName: {f['displayName']}, downloadUrl: {f.get('downloadUrl')}")
