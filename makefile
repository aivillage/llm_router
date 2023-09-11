up:
	docker compose -f dockerfiles/docker-compose.dev.yml up --build

test:
	docker compose -f tests/docker-compose.yml run llm_router_test

publish_dev:
	docker build --file dockerfiles/Dockerfile -t aivillage/llm_router:dev . && \
	docker push aivillage/llm_router:dev