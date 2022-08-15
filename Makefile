# Copyright 2019 Jeremy Edwards
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

include proto.mk

GO = GO111MODULE=on go
DOCKER = DOCKER_CLI_EXPERIMENTAL=enabled docker

BASE_VERSION = 0.0.0-dev
SHORT_SHA = $(shell git rev-parse --short=7 HEAD | tr -d [:punct:])
VERSION_SUFFIX = $(SHORT_SHA)
VERSION = $(BASE_VERSION)-$(VERSION_SUFFIX)
BUILD_DATE = $(shell date -u +'%Y-%m-%dT%H:%M:%SZ')
TAG := v$(VERSION)

export PATH := $(PWD)/build/toolchain/bin:$(PATH):/root/go/bin:/usr/local/go/bin:/usr/go/bin

REGISTRY = docker.io/jeremyje
CLEANUP_IMAGE = $(REGISTRY)/cleanup
RMLIST_IMAGE = $(REGISTRY)/rmlist
SIMILAR_IMAGE = $(REGISTRY)/similar
UNIQUE_IMAGE = $(REGISTRY)/unique
ALL_IMAGES = $(CLEANUP_IMAGE) $(RMLIST_IMAGE) $(SIMILAR_IMAGE) $(UNIQUE_IMAGE)

PROTOS = internal/metadata/proto/metadata.pb.go

ASSETS = $(PROTOS)
NICHE_PLATFORMS = freebsd openbsd netbsd darwin
LINUX_PLATFORMS = linux_386 linux_amd64 linux_arm_v5 linux_arm_v6 linux_arm_v7 linux_arm64 linux_s390x linux_ppc64le linux_riscv64 linux_mips64le linux_mips linux_mipsle linux_mips64
LINUX_NICHE_PLATFORMS = 
WINDOWS_PLATFORMS = windows_386 windows_amd64
MAIN_PLATFORMS = windows_amd64 linux_amd64 linux_arm64
ALL_PLATFORMS = $(LINUX_PLATFORMS) $(LINUX_NICHE_PLATFORMS) $(WINDOWS_PLATFORMS) $(foreach niche,$(NICHE_PLATFORMS),$(niche)_amd64 $(niche)_arm64)
ALL_APPS = cleanup rmlist similar unique

MAIN_BINARIES = $(foreach app,$(ALL_APPS),$(foreach platform,$(MAIN_PLATFORMS),build/bin/$(platform)/$(app)$(if $(findstring windows_,$(platform)),.exe,)))
ALL_BINARIES = $(foreach app,$(ALL_APPS),$(foreach platform,$(ALL_PLATFORMS),build/bin/$(platform)/$(app)$(if $(findstring windows_,$(platform)),.exe,)))
# https://hub.docker.com/_/microsoft-windows-nanoserver
WINDOWS_VERSIONS = 1809 20H2 ltsc2022
BUILDX_BUILDER = buildx-builder
LINUX_CPU_PLATFORMS = amd64 arm64 ppc64le s390x arm/v5 arm/v6 arm/v7

binaries: $(MAIN_BINARIES)
all: $(ALL_BINARIES)
assets: $(ASSETS)
protos: $(PROTOS)

build/bin/%: $(ASSETS)
	GOOS=$(firstword $(subst _, ,$(notdir $(abspath $(dir $@))))) GOARCH=$(word 2, $(subst _, ,$(notdir $(abspath $(dir $@))))) GOARM=$(subst v,,$(word 3, $(subst _, ,$(notdir $(abspath $(dir $@)))))) CGO_ENABLED=0 $(GO) build -o $@ cmd/$(basename $(notdir $@))/$(basename $(notdir $@)).go
	touch $@

# https://github.com/docker-library/official-images#architectures-other-than-amd64
images: DOCKER_PUSH = --push
images: linux-images windows-images
	-$(DOCKER) manifest rm $(CLEANUP_IMAGE):$(TAG)
	-$(DOCKER) manifest rm $(RMLIST_IMAGE):$(TAG)
	-$(DOCKER) manifest rm $(SIMILAR_IMAGE):$(TAG)
	-$(DOCKER) manifest rm $(UNIQUE_IMAGE):$(TAG)

	for image in $(ALL_IMAGES) ; do \
		$(DOCKER) manifest create $$image:$(TAG) $(foreach winver,$(WINDOWS_VERSIONS),$${image}:$(TAG)-windows_amd64-$(winver)) $(foreach platform,$(LINUX_PLATFORMS),$${image}:$(TAG)-$(platform)) ; \
		for winver in $(WINDOWS_VERSIONS) ; do \
			windows_version=`$(DOCKER) manifest inspect mcr.microsoft.com/windows/nanoserver:$${winver} | jq -r '.manifests[0].platform["os.version"]'`; \
			$(DOCKER) manifest annotate --os-version $${windows_version} $${image}:$(TAG) $${image}:$(TAG)-windows_amd64-$${winver} ; \
		done ; \
		$(DOCKER) manifest push $$image:$(TAG) ; \
	done

ensure-builder:
	-$(DOCKER) buildx create --name $(BUILDX_BUILDER)

ALL_LINUX_IMAGES = $(foreach app,$(ALL_APPS),$(foreach platform,$(LINUX_PLATFORMS),linux-image-$(app)-$(platform)))
linux-images: $(ALL_LINUX_IMAGES)

linux-image-cleanup-%: build/bin/%/cleanup ensure-builder
	$(DOCKER) buildx build --builder $(BUILDX_BUILDER) --platform $(subst _,/,$*) --build-arg BINARY_PATH=$< -f cmd/cleanup/Dockerfile -t $(CLEANUP_IMAGE):$(TAG)-$* . $(DOCKER_PUSH)

linux-image-rmlist-%: build/bin/%/rmlist ensure-builder
	$(DOCKER) buildx build --builder $(BUILDX_BUILDER) --platform $(subst _,/,$*) --build-arg BINARY_PATH=$< -f cmd/rmlist/Dockerfile -t $(RMLIST_IMAGE):$(TAG)-$* . $(DOCKER_PUSH)

linux-image-similar-%: build/bin/%/similar ensure-builder
	$(DOCKER) buildx build --builder $(BUILDX_BUILDER) --platform $(subst _,/,$*) --build-arg BINARY_PATH=$< -f cmd/similar/Dockerfile -t $(SIMILAR_IMAGE):$(TAG)-$* . $(DOCKER_PUSH)

linux-image-unique-%: build/bin/%/unique ensure-builder
	$(DOCKER) buildx build --builder $(BUILDX_BUILDER) --platform $(subst _,/,$*) --build-arg BINARY_PATH=$< -f cmd/unique/Dockerfile -t $(UNIQUE_IMAGE):$(TAG)-$* . $(DOCKER_PUSH)

ALL_WINDOWS_IMAGES = $(foreach app,$(ALL_APPS),$(foreach winver,$(WINDOWS_VERSIONS),windows-image-$(app)-$(winver)))
windows-images: $(ALL_WINDOWS_IMAGES)

windows-image-cleanup-%: build/bin/windows_amd64/cleanup.exe ensure-builder
	$(DOCKER) buildx build --builder $(BUILDX_BUILDER) --platform windows/amd64 -f cmd/cleanup/Dockerfile.windows --build-arg WINDOWS_VERSION=$* -t $(CLEANUP_IMAGE):$(TAG)-windows_amd64-$* . $(DOCKER_PUSH)

windows-image-rmlist-%: build/bin/windows_amd64/rmlist.exe ensure-builder
	$(DOCKER) buildx build --builder $(BUILDX_BUILDER) --platform windows/amd64 -f cmd/rmlist/Dockerfile.windows --build-arg WINDOWS_VERSION=$* -t $(RMLIST_IMAGE):$(TAG)-windows_amd64-$* . $(DOCKER_PUSH)

windows-image-similar-%: build/bin/windows_amd64/similar.exe ensure-builder
	$(DOCKER) buildx build --builder $(BUILDX_BUILDER) --platform windows/amd64 -f cmd/similar/Dockerfile.windows --build-arg WINDOWS_VERSION=$* -t $(SIMILAR_IMAGE):$(TAG)-windows_amd64-$* . $(DOCKER_PUSH)

windows-image-unique-%: build/bin/windows_amd64/unique.exe ensure-builder
	$(DOCKER) buildx build --builder $(BUILDX_BUILDER) --platform windows/amd64 -f cmd/unique/Dockerfile.windows --build-arg WINDOWS_VERSION=$* -t $(UNIQUE_IMAGE):$(TAG)-windows_amd64-$* . $(DOCKER_PUSH)

deps:
	$(GO) mod download

fmt: $(ASSETS)
	gofmt -s -w cmd/ internal/ testdata/
	$(GO) fmt ./...

vet: $(ASSETS)
	$(GO) vet ./...

lint: fmt vet

test: $(ASSETS)
	$(GO) test ./... -race -cover -timeout=10s

bench: $(ASSETS)
	$(GO) test ./... -bench=.

check: lint test bench

clean:
	rm -f coverage.txt
	-chmod -R +w $(BUILD_DIR)
	rm -rf $(BUILD_DIR)
	rm -f $(PROTOS)

coverage.txt:
	touch coverage.txt
	for pkg in $(shell go list ./... | grep -v vendor | grep -v go:); do \
		$(GO) test -race -coverprofile=profile.out -covermode=atomic "$$pkg"; \
			touch profile.out ; \
			cat profile.out >> coverage.txt ; \
			rm profile.out ; \
	done

presubmit: clean check all coverage.txt

.PHONY: all deps fmt vet lint test bench check clean presubmit
