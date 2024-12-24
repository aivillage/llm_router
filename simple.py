import requests
from uuid import uuid4

url = "http://0.0.0.0:8000"


def generate_for_model(model: str):
    payload = {
        "uuid": str(uuid4()),
        "prompt": "What is going on?",
        "model": model,
    }
    response = requests.post(url + "/chat/generate", json=payload)
    generation = response.json()["generation"]
    print(generation)

print("======== zephyr-7b =========")
generate_for_model("zephyr-7b")
print("======== falcon-7b =========")
generate_for_model("falcon-7b")