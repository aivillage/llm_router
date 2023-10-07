import pytest
import requests
import json
from uuid import uuid4

url = "http://llm_router:8000"


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

    history = [{"prompt": "test", "generation": generation}]

    payload = {
        "uuid": str(uuid4()),
        "prompt": "test",
        "preprompt": "test",
        "model": model,
        "history": history,
    }
    response = requests.post(url + "/chat/generate", json=payload)
    assert response.status_code == 200
    generation = response.json()["generation"]
    assert generation != ""


@pytest.mark.external
def test_generate_huggingface():
    generate_for_model("oasst-pythia-12b")


@pytest.mark.external
def test_generate_openai():
    generate_for_model("gpt-3.5-turbo")
