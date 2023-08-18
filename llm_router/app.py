from typing import Optional, List
from fastapi import FastAPI
from pydantic import BaseModel
from starlette.status import HTTP_404_NOT_FOUND, HTTP_422_UNPROCESSABLE_ENTITY
from logging import getLogger
from .constants import RedisLocal
log = getLogger("generator")

from .models.models import models

app = FastAPI()


class GenerateRequest(BaseModel):
    # This needs to be unique to the request, it is used to make requests idempotent
    uuid: str
    prompt: str
    preprompt: Optional[str] = None
    model: str


class GenerateResponse(BaseModel):
    uuid: str
    generation: str = ""


@app.post("/generate")
async def generate(request: GenerateRequest) -> GenerateResponse:
    # Check redis cache:
    cached = await RedisLocal.get(request.uuid)
    if cached is not None:
        if cached == "ERROR":
            return HTTP_422_UNPROCESSABLE_ENTITY(generation="", error="Error in previous generation")
        return GenerateResponse(uuid=request.uuid, generation=cached)
    
    if request.model not in models:
        return HTTP_404_NOT_FOUND("Model not found")
    
    try:
        generation = await models[request.model](request.prompt, request.preprompt)
        # Cache the result
        await RedisLocal.set(request.uuid, generation)
        return GenerateResponse(uuid=request.uuid, generation=generation)
    except Exception as e:
        log.exception(e)
        await RedisLocal.set(request.uuid, "ERROR")
        return HTTP_422_UNPROCESSABLE_ENTITY(generation="", error=str(e))
    

class ModelsResponse(BaseModel):
    models: List[str] = []


@app.get("/models")
async def get_models() -> ModelsResponse:
    return ModelsResponse(models=list(models.keys()))


@app.get("/health")
async def health() -> str:
    return "OK"


