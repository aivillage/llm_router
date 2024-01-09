import requests
from uuid import uuid4

url = "http://0.0.0.0:8000"


def generate_for_model(model: str):
    payload = {
        "uuid": str(uuid4()),
        "prompt": "test",
        "preprompt": "test",
        "model": model,
    }
    response = requests.post(url + "/chat/generate", json=payload)
    assert response.status_code == 200
    generation = response.json()["generation"]
    assert generation != ""

generate_for_model("zephyr-7b")