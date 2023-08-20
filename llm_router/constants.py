import os
import aioredis
from logging import getLogger
log = getLogger("generator")

MODEL_DIR = os.getenv("MODEL_DIR", "/opt/models/")

def load_redis():
    REDIS_URL = os.getenv("REDIS_URL")
    if REDIS_URL is None:
        log.info("REDIS_URL not set, using default")
        return None
    log.info(f"Connecting to redis at {REDIS_URL}")
    REDIS_USERNAME = os.getenv("REDIS_USERNAME")
    if REDIS_USERNAME is None:
        raise ValueError("REDIS_USERNAME not set")
    REDIS_PASSWORD = os.getenv("REDIS_PASSWORD")
    if REDIS_PASSWORD is None:
        raise ValueError("REDIS_PASSWORD not set")

    return aioredis.from_url(REDIS_URL, username=REDIS_USERNAME, password=REDIS_PASSWORD, decode_responses=True)
