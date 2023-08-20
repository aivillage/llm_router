import os, json
from typing import Optional
from fastapi import HTTPException
from starlette.status import HTTP_429_TOO_MANY_REQUESTS
import aiohttp
from logging import getLogger
from ..secrets import hf_key
log = getLogger("generator")


def load_huggingface_models(file):
    with open(file) as f:
        models = json.load(f)

    for model, params in models.items():
        yield model, huggingface_single_model_factory(params['url'], params['parameters'], params['prompt_format'])


def huggingface_single_model_factory(url: str, parameters: dict, prompt_format: str):
    async def generate(prompt: str, preprompt: Optional[str] = None) -> str:
        HEADERS = {'Authorization': f'Bearer {hf_key()}'}
        full_prompt = prompt_format.format(preprompt=preprompt, prompt=prompt)
        payload = {'inputs': full_prompt, "parameters" : parameters, "stream": False}
        
        async with aiohttp.ClientSession() as session:
            async with session.post(
                    url=url,
                    headers=HEADERS,
                    json=payload) as raw_response:
                json_response = await raw_response.json()
                if 'error' in json_response:
                    log.error(f"Error generating: {json_response['error']}")
                    raise HTTPException(
                        HTTP_429_TOO_MANY_REQUESTS, "Too Many Requests", headers={"Retry-After": str(1000)}
                    )
        generated_text = json_response[0]['generated_text']
        return generated_text

    return generate