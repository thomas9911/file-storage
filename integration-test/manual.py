import os
import mimetypes
import requests
from tqdm.auto import tqdm
from pathlib import Path

URL = "http://localhost:3030/api/basic"

r = requests.post(URL + "/test")
print(r.text)
print(r.status_code)
file_path = Path(r"/home/thomas/Downloads/ubuntu-22.04.1-desktop-amd64.iso")
total_length = file_path.stat().st_size

# with open(r"C:\Users\Thomas\Downloads\Unofficial Mass Effect 2 Legendary Edition Patch-8-0-9-2-1661103809.7z", "rb") as f:
(content_type, _) = mimetypes.guess_type(file_path)
with open(file_path, "rb") as f:
    with tqdm.wrapattr(f, "read", total=total_length, desc="") as raw:
        r = requests.post(
            URL + "/test/tmp02" + file_path.name,
            data=raw,
            headers={"content-type": content_type},
        )

print(r.text)
print(r.status_code)
