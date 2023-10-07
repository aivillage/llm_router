import requests
import json
from uuid import uuid4

with open("mock.json") as f:
    mock_models = json.load(f)

short_response = mock_models["mock_model"]["short"]
long_response = mock_models["mock_model"]["long"]

url = "http://llm_router:8000"


def test_modes():
    """Checks that the list of models is returned correctly"""
    mock_model_names = set(mock_models.keys())

    response = requests.get(url + "/chat/models")
    assert response.status_code == 200
    print(response.json())

    llm_router_returned_names = set(response.json()["models"])

    assert mock_model_names.issubset(llm_router_returned_names)


def test_generate():
    """Test that that llm router """
    uuid = str(uuid4())
    payload = {"uuid": uuid, "prompt": "test", "system": "test", "model": "mock_model"}
    response = requests.post(url + "/chat/generate", json=payload)
    assert response.status_code == 200
    assert response.json()["generation"] == f"response: 0, {short_response}"


def test_generate_error():
    uuid = str(uuid4())
    payload = {"uuid": uuid, "prompt": "error", "system": "test", "model": "mock_model"}
    response = requests.post(url + "/chat/generate", json=payload)
    assert response.status_code == 500
    response = requests.post(url + "/chat/generate", json=payload)
    assert response.status_code == 500


def test_generate_long():
    uuid = str(uuid4())
    payload = {
        "uuid": uuid,
        "prompt": "prompt_too_long",
        "system": "test",
        "model": "mock_model",
    }
    response = requests.post(url + "/chat/generate", json=payload)
    assert response.status_code == 422
    response = requests.post(url + "/chat/generate", json=payload)
    assert response.status_code == 422


def test_generate_long_response():
    uuid = str(uuid4())
    payload = {
        "uuid": uuid,
        "prompt": "long_response",
        "system": "test",
        "model": "mock_model",
    }
    response = requests.post(url + "/chat/generate", json=payload)
    assert response.status_code == 200
    assert response.json()["generation"].endswith(long_response)


def test_generate_model_not_found():
    uuid = str(uuid4())
    payload = {"uuid": uuid, "prompt": "test", "system": "test", "model": "not_found"}
    response = requests.post(url + "/chat/generate", json=payload)
    assert response.status_code == 404
    response = requests.post(url + "/chat/generate", json=payload)
    assert response.status_code == 404


def test_generate_cache():
    """Tests caching. Expected behavior is that response is the same"""
    uuid = str(uuid4())
    payload = {"uuid": uuid, "prompt": "test", "system": "test", "model": "mock_model"}
    
    for _ in range(2):
        response = requests.post(url + "/chat/generate", json=payload)
        assert response.status_code == 200
        assert response.json()["generation"].startswith("response: 0,")
        assert response.json()["generation"].endswith(short_response)


def test_generate_chat():
    uuid = str(uuid4())
    payload = {"uuid": uuid, "prompt": "test", "system": "test", "model": "mock_model"}
    response = requests.post(url + "/chat/generate", json=payload)
    history = [{"prompt": "test", "generation": response.json()["generation"]}]
    uuid = str(uuid4())
    payload = {
        "uuid": uuid,
        "prompt": "test",
        "system": "test",
        "model": "mock_model",
        "history": history,
    }
    response = requests.post(url + "/chat/generate", json=payload)
    assert response.status_code == 200
    assert response.json()["generation"].startswith("response: 1,")
    assert response.json()["generation"].endswith(short_response)
    response = requests.post(url + "/chat/generate", json=payload)
    assert response.status_code == 200
    assert response.json()["generation"].startswith("response: 1,")
    assert response.json()["generation"].endswith(short_response)
