'''
For testing
'''
import os, json
from typing import Optional
from fastapi import HTTPException
from starlette.status import HTTP_422_UNPROCESSABLE_ENTITY
from logging import getLogger
log = getLogger("generator")


def load_mock_models(file):
    with open(file) as f:
        models = json.load(f)

    for model_name, responses in models.items():
        yield model_name, generate_mock(responses['short_response'], responses['long_response'])


def generate_mock(short_response: str, long_response: str) -> str:
    async def mock_generate(prompt: str, preprompt: Optional[str] = None) -> str:
        if preprompt is not None:
            prompt = preprompt + "\n" + prompt
        if "long_response":
            return long_response
        if "long_prompt" in prompt:
            raise HTTPException(HTTP_422_UNPROCESSABLE_ENTITY, "Prompt too long")
        if "error" in prompt:
            raise HTTPException(HTTP_422_UNPROCESSABLE_ENTITY, "Error in prompt")
        return short_response
    
    return mock_generate