# import pytest
import requests

URL = "http://localhost:3030/api/basic"


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
