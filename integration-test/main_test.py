# import pytest
import requests
import jwt
from datetime import datetime
from datetime import timedelta

URL = "http://localhost:3030/api/basic"


def admin_jwt(method, path):
    return jwt.encode(
        {
            "jti": "123456",
            "sub": "username",
            "path": path,
            "method": method,
            "exp": datetime.utcnow() + timedelta(minutes=5),
            "nbf": datetime.utcnow(),
        },
        "secret",
        algorithm="HS256",
    )


def test_create_bucket():
    url = URL + "/my_bucket"
    requests.delete(url)
    r = requests.post(url)

    assert 200 == r.status_code
    assert {"bucket": "my_bucket", "created": True, "info": "OK"} == r.json()

    r = requests.post(url)

    assert 409 == r.status_code
    assert {
        "bucket": "my_bucket",
        "created": False,
        "info": "Bucket already exists",
    } == r.json()


def test_delete_bucket():
    url = URL + "/my_delete_bucket"
    requests.post(url)
    r = requests.delete(url)

    assert 200 == r.status_code
    assert {"bucket": "my_delete_bucket", "info": "OK"} == r.json()

    r = requests.delete(url)

    assert 200 == r.status_code
    assert {"bucket": "my_delete_bucket", "info": "OK"} == r.json()


def test_create_object():
    url = URL + "/my_object_bucket"
    requests.delete(url + "?purge=true")
    r = requests.post(url)
    assert 200 == r.status_code

    with open("image.jpg", "rb") as f:
        r = requests.post(
            url + "/image.jpg", data=f, headers={"content-type": "image/jpeg"}
        )
    assert 200 == r.status_code
    assert {
        "bucket": "my_object_bucket",
        "info": "OK",
        "created": True,
        "filename": "image.jpg",
    } == r.json()

    with open("image.jpg", "rb") as f:
        r = requests.post(
            url + "/image.jpg", data=f, headers={"content-type": "image/jpeg"}
        )
    assert 409 == r.status_code
    assert {
        "bucket": "my_object_bucket",
        "info": "Object already exists",
        "created": False,
        "filename": "image.jpg",
    } == r.json()

    r = requests.delete(url)

    assert 400 == r.status_code
    assert {"bucket": "my_object_bucket", "info": "bucket is not empty"} == r.json()

    r = requests.delete(url + "/image.jpg")

    assert 200 == r.status_code
    assert {
        "bucket": "my_object_bucket",
        "filename": "image.jpg",
        "info": "OK",
    } == r.json()

    r = requests.delete(url + "/image.jpg")

    assert 404 == r.status_code
    assert {
        "bucket": "my_object_bucket",
        "filename": "image.jpg",
        "info": "object not found",
    } == r.json()

    r = requests.delete(url + "?purge=true")

    assert 200 == r.status_code
    assert {"bucket": "my_object_bucket", "info": "OK"} == r.json()


def test_login_create_object():
    url = URL + "/my_object_bucket"

    jwt = admin_jwt("DELETE", "my_object_bucket")
    requests.delete(url + "?purge=true", headers={"authorization": f"bearer {jwt}"})

    jwt = admin_jwt("POST", "my_object_bucket")
    r = requests.post(url, headers={"authorization": f"bearer {jwt}"})
    assert 200 == r.status_code

    jwt = admin_jwt("POST", "my_object_bucket/image.jpg")
    with open("image.jpg", "rb") as f:
        r = requests.post(
            url + "/image.jpg",
            data=f,
            headers={"content-type": "image/jpeg", "authorization": f"bearer {jwt}"},
        )
    assert 200 == r.status_code
    assert {
        "bucket": "my_object_bucket",
        "info": "OK",
        "created": True,
        "filename": "image.jpg",
    } == r.json()
