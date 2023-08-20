'''
This should be replaced by a database that allows for dynamic loading of models and handling of errors.
'''

import os
from .huggingface import load_huggingface_models
from .mock import load_mock_models
from ..constants import MODEL_DIR

from logging import getLogger
log = getLogger("generator")


def load_models():
    models = {}
    for file in os.listdir(MODEL_DIR):
        if file == "mock.json":
            log.info("Loading mock models")
            for model, generate in load_mock_models(os.path.join(MODEL_DIR, file)):
                models[model] = generate
        if file == "huggingface.json":
            log.info("Loading huggingface models")
            for model, generate in load_huggingface_models(os.path.join(MODEL_DIR, file)):
                models[model] = generate
    return models

models = load_models()