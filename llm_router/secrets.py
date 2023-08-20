import os


def hf_key():
    HF_KEY = os.environ.get("HUGGINGFACE_API_KEY")
    if HF_KEY is None:
        raise ValueError("HUGGINGFACE_API_KEY not set")
    return HF_KEY