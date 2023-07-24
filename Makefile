# Project makefile
#
# I'm unable to get docker to build my project with a single command,
# so it's makefile time.

image_name = ntu_canoebot
manifest = $(shell find -name "Cargo.*" -type f)

default: build

cache:
	docker build -t ntu_canoebot_cache -f docker/cache.Dockerfile .

build: cache
	docker compose build

up: build
	docker compose up -d

down:
	docker compose down

save: build
	docker save --output $(image_name).tar $(image_name)

info:
	$(info $$manifest is [${manifest}])
