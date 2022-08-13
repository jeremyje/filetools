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

GO = GO111MODULE=on go
GO_BUILD = GO111MODULE=on CGO_ENABLED=0 go build -a -installsuffix cgo

ifeq ($(OS),Windows_NT)
	DOT_EXE = .exe
else
	DOT_EXE =
endif

REGISTRY = docker.io/jeremyje
CLEANUP_IMAGE = $(REGISTRY)/cleanup
SIMILAR_IMAGE = $(REGISTRY)/similar
UNIQUE_IMAGE = $(REGISTRY)/unique


ASSETS = 
NICHE_PLATFORMS = freebsd openbsd netbsd darwin
LINUX_PLATFORMS = linux_386 linux_amd64 linux_arm_v5 linux_arm_v6 linux_arm_v7 linux_arm64 linux_s390x linux_ppc64le linux_riscv64 linux_mips64le linux_mips linux_mipsle linux_mips64
LINUX_NICHE_PLATFORMS = 
WINDOWS_PLATFORMS = windows_386 windows_amd64
MAIN_PLATFORMS = windows_amd64 linux_amd64 linux_arm64
ALL_PLATFORMS = $(LINUX_PLATFORMS) $(LINUX_NICHE_PLATFORMS) $(WINDOWS_PLATFORMS) $(foreach niche,$(NICHE_PLATFORMS),$(niche)_amd64 $(niche)_arm64)
ALL_APPS = cleanup similar unique

MAIN_BINARIES = $(foreach app,$(ALL_APPS),$(foreach platform,$(MAIN_PLATFORMS),build/bin/$(platform)/$(app)$(if $(findstring windows_,$(platform)),.exe,)))
ALL_BINARIES = $(foreach app,$(ALL_APPS),$(foreach platform,$(ALL_PLATFORMS),build/bin/$(platform)/$(app)$(if $(findstring windows_,$(platform)),.exe,)))
# https://hub.docker.com/_/microsoft-windows-nanoserver
WINDOWS_VERSIONS = 1809 20H2 ltsc2022
BUILDX_BUILDER = buildx-builder
LINUX_CPU_PLATFORMS = amd64 arm64 ppc64le s390x arm/v5 arm/v6 arm/v7


binaries: $(MAIN_BINARIES)
all: $(ALL_BINARIES)

build/bin/%: $(ASSETS)
	GOOS=$(firstword $(subst _, ,$(notdir $(abspath $(dir $@))))) GOARCH=$(word 2, $(subst _, ,$(notdir $(abspath $(dir $@))))) GOARM=$(subst v,,$(word 3, $(subst _, ,$(notdir $(abspath $(dir $@)))))) CGO_ENABLED=0 $(GO) build -o $@ cmd/$(basename $(notdir $@))/$(basename $(notdir $@)).go
	touch $@

ALL_LINUX_IMAGES = $(foreach app,$(ALL_APPS),$(foreach platform,$(LINUX_PLATFORMS),linux-image-$(app)-$(platform)))
linux-images: $(ALL_LINUX_IMAGES)

linux-image-cleanup-%: build/bin/%/cleanup ensure-builder
	$(DOCKER) buildx build --builder $(BUILDX_BUILDER) --platform $(subst _,/,$*) --build-arg BINARY_PATH=$< -f cmd/cleanup/Dockerfile -t $(CLEANUP_IMAGE):$(TAG)-$* . $(DOCKER_PUSH)

ALL_WINDOWS_IMAGES = $(foreach app,$(ALL_APPS),$(foreach winver,$(WINDOWS_VERSIONS),windows-image-$(app)-$(winver)))
windows-images: $(ALL_WINDOWS_IMAGES)

windows-image-cleanup-%: build/bin/windows_amd64/cleanup.exe ensure-builder
	$(DOCKER) buildx build --builder $(BUILDX_BUILDER) --platform windows/amd64 -f cmd/cleanup/Dockerfile.windows --build-arg WINDOWS_VERSION=$* -t $(CLEANUP_IMAGE):$(TAG)-windows_amd64-$* . $(DOCKER_PUSH)

ensure-builder:
	-$(DOCKER) buildx create --name $(BUILDX_BUILDER)

deps:
	$(GO) mod download

fmt:
	gofmt -s -w .
	$(GO) fmt ./...

vet:
	$(GO) vet ./...

lint: fmt vet

test:
	$(GO) test ./... -race -cover -timeout=10s

bench:
	$(GO) test ./... -bench=.

check: lint test bench

clean:
	rm -f coverage.txt
	rm -f cmd/cleanup/cleanup$(DOT_EXE)
	rm -f cmd/similar/similar$(DOT_EXE)
	rm -f cmd/unique/unique$(DOT_EXE)

target:
	for number in 1 2 3 4 ; do \
		echo $$number ; \
	done

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
