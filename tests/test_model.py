import requests
import json
from uuid import uuid4

url = "http://llm_router:8000"

def test_generate_huggingface():
    uuid = str(uuid4())
    payload = {"uuid": uuid, "prompt": "test", "preprompt": "test", "model": "oasst-pythia-12b"}
    response = requests.post(url + "/chat", json=payload)
    assert response.status_code == 200
    assert response.json()["generation"] != ""