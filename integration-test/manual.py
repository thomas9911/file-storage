import requests

URL = "http://localhost:3030/api/basic"

r = requests.post(URL + "/test")
print(r.text)
print(r.status_code)

with open(r"C:\Users\Thomas\Downloads\Unofficial Mass Effect 2 Legendary Edition Patch-8-0-9-2-1661103809.7z", "rb") as f:
    r = requests.post(URL + "/test/large.7z", data=f, headers={"content-type": "application/octet-stream"})

print(r.text)
print(r.status_code)
