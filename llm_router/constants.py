import os
from dataclasses import dataclass
import aioredis

HF_KEY = os.environ.get("HUGGINGFACE_API_KEY")
if HF_KEY is None:
    raise ValueError("HUGGINGFACE_API_KEY not set")

HF_URL = os.getenv("HUGGINGFACE_API_URL", "https://api-inference.huggingface.co/models")
MODEL_DIR = os.getenv("MODEL_DIR", "/opt/models/")


REDIS_URL = os.getenv("REDIS_URL")
if REDIS_URL is None:
    raise ValueError("REDIS_URL not set")
REDIS_USERNAME = os.getenv("REDIS_USERNAME")
if REDIS_USERNAME is None:
    raise ValueError("REDIS_USERNAME not set")
REDIS_PASSWORD = os.getenv("REDIS_PASSWORD")
if REDIS_PASSWORD is None:
    raise ValueError("REDIS_PASSWORD not set")

RedisLocal = aioredis.from_url(REDIS_URL, username=REDIS_USERNAME, password=REDIS_PASSWORD, decode_responses=True)
