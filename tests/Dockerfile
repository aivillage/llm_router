FROM python:3.10-slim-bookworm as release
WORKDIR /opt/llm_router_tests

COPY --chown=1001:1001 . /opt/llm_router_tests/
RUN pip install --no-cache-dir -r requirements.txt
RUN apt update && apt install -y curl

ENTRYPOINT ["pytest"]
