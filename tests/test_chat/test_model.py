import requests
import json
from uuid import uuid4

url = "http://llm_router:8000"

def test_generate_huggingface():
    payload = {"uuid": str(uuid4()), "prompt": "test", "preprompt": "test", "model": "oasst-pythia-12b"}
    response = requests.post(url + "/chat/generate", json=payload)
    assert response.status_code == 200
    generation = response.json()["generation"]
    assert generation != ""

    history = [{"prompt": "test", "generation": generation}]

    payload = {"uuid": str(uuid4()), "prompt": "test", "preprompt": "test", "model": "oasst-pythia-12b", "history": history}
    response = requests.post(url + "/chat/generate", json=payload)
    assert response.status_code == 200
    generation = response.json()["generation"]
    assert generation != ""
