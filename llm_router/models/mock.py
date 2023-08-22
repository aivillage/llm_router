'''
For testing
'''
import os, json
from typing import Optional
from fastapi import HTTPException
from starlette.status import HTTP_422_UNPROCESSABLE_ENTITY, HTTP_500_INTERNAL_SERVER_ERROR, HTTP_429_TOO_MANY_REQUESTS
from logging import getLogger
log = getLogger("generator")


def load_mock_models(file):
    with open(file) as f:
        models = json.load(f)

    for model_name, responses in models.items():
        log.info(f"Loading mock model {model_name}")
        yield model_name, generate_mock(responses['short_response'], responses['long_response'])


def generate_mock(short_response: str, long_response: str) -> str:
    async def mock_generate(prompt: str, preprompt: Optional[str] = None) -> str:
        if preprompt is not None:
            prompt = preprompt + "\n" + prompt
        if "long_response" in prompt:
            return long_response
        if "long_prompt" in prompt:
            raise HTTPException(HTTP_422_UNPROCESSABLE_ENTITY, "Prompt too long")
        if "too_many_requests" in prompt:
            raise HTTPException(HTTP_429_TOO_MANY_REQUESTS, "Too many requests")
        if "error" in prompt:
            raise HTTPException(HTTP_500_INTERNAL_SERVER_ERROR, "Error in prompt")
        return short_response
    
    return mock_generate