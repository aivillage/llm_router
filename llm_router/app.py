import json
from typing import Optional, List
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from starlette.status import HTTP_404_NOT_FOUND, HTTP_422_UNPROCESSABLE_ENTITY, HTTP_500_INTERNAL_SERVER_ERROR
from logging import getLogger
from .constants import load_redis
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
    generation: str

RedisLocal = load_redis()

async def generate_cacheless(request: GenerateRequest) -> str:
    if request.model not in models:
        raise HTTPException(HTTP_404_NOT_FOUND, "Model not found")
    
    try:
        generation = await models[request.model](request.prompt, request.preprompt)
        return generation
    except HTTPException as e:
        raise e
    except Exception as e:
        raise HTTPException(HTTP_500_INTERNAL_SERVER_ERROR, str(e))

@app.post("/generate")
async def generate(request: GenerateRequest) -> GenerateResponse:
    # Check redis cache:
    if RedisLocal is not None:
        cached = await RedisLocal.get(request.uuid)
        if cached is not None:
            if cached.startswith("<ERROR>"):
                error_json = json.loads(cached[7:])
                raise HTTPException(error_json["code"], error_json["message"])
                
            return GenerateResponse(uuid=request.uuid, generation=cached)
        if cached is None:
            try:
                generation = await generate_cacheless(request)
                await RedisLocal.set(request.uuid, generation)
                return GenerateResponse(uuid=request.uuid, generation=generation)
            except HTTPException as e:
                error_json = {"code": e.status_code, "message": e.detail}
                await RedisLocal.set(request.uuid, f"<ERROR>{json.dumps(error_json)}")
                raise e
    else:
        generation = await generate_cacheless(request)
        return GenerateResponse(uuid=request.uuid, generation=generation)
    

class ModelsResponse(BaseModel):
    models: List[str] = []


@app.get("/models")
async def get_models() -> ModelsResponse:
    return ModelsResponse(models=list(models.keys()))


@app.get("/health")
async def health() -> str:
    return "OK"


