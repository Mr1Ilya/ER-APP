import urllib.request
import json

api_key = "$2a$10$bL4bIL5pUWqfcO7KQtnMReakwtfHbNKh6v1uTpKlzhwoueEJQnPnm"
url = "https://api.curseforge.com/v1/mods/search?gameId=432&classId=4471&pageSize=5&sortField=2&sortOrder=desc"
req = urllib.request.Request(url, headers={'x-api-key': api_key, 'Accept': 'application/json'})
try:
    with urllib.request.urlopen(req) as resp:
        data = json.loads(resp.read().decode('utf-8'))
        print("Success! Hits:", len(data.get('data', [])))
        for item in data.get('data', []):
            print(f"- {item['id']}: {item['name']}")
except Exception as e:
    print("Error:", e)
