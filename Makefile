# axon — KubePivot 多语言项目 Makefile

ROOT_DIR := $(shell pwd)
VERSION ?= v0.1.0
ARCH ?= amd64
REGISTRY_PREFIX ?= qingchun22
PROJECT_NAME := axon
MODULE_PATH := axon
DOCKERFILE := build/docker/$(PROJECT_NAME)/Dockerfile

include scripts/make-rules/deploy.mk

.PHONY: deploy.build deploy.push
deploy.build:
	@docker build -t $(REGISTRY_PREFIX)/$(PROJECT_NAME)-$(ARCH):$(VERSION) -f $(DOCKERFILE) .

deploy.push:
	@docker push $(REGISTRY_PREFIX)/$(PROJECT_NAME)-$(ARCH):$(VERSION)
