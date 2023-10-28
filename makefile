up:
	docker compose -f dockerfiles/docker-compose.dev.yml up --build

build_test:
	docker compose -f tests/docker-compose.yml build 

test:
	docker compose -f tests/docker-compose.yml run llm_router_test

publish_dev:
	docker buildx build --platform linux/amd64,linux/arm64,linux/arm/v7 --file dockerfiles/Dockerfile -t aivillage/llm_router:dev --push .

lockfile:
	cargo generate-lockfile