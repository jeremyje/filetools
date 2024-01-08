# Copyright 2024 Jeremy Edwards
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

CARGO = cargo
DOCKER = DOCKER_CLI_EXPERIMENTAL=enabled docker

BASE_VERSION = 0.0.0-dev
SHORT_SHA = $(shell git rev-parse --short=7 HEAD | tr -d [:punct:])
VERSION_SUFFIX = $(SHORT_SHA)
VERSION = $(BASE_VERSION)-$(VERSION_SUFFIX)
BUILD_DATE = $(shell date -u +'%Y-%m-%dT%H:%M:%SZ')
TAG := v$(VERSION)

REGISTRY = docker.io/jeremyje
FILETOOL_IMAGE = $(REGISTRY)/filetool
ALL_IMAGES = $(FILETOOL_IMAGE)

PROTOS = 

ASSETS = $(PROTOS)
ALL_APPS = filetool
BUILD_TYPES = debug release

# rustc --print target-list
RUST_TRIPLES = x86_64-unknown-linux-gnu
RUST_TRIPLES += x86_64-pc-windows-gnu
#RUST_TRIPLES += aarch64-apple-darwin
#RUST_TRIPLES += aarch64-pc-windows-msvc
#RUST_TRIPLES += aarch64-unknown-linux-musl
#RUST_TRIPLES += x86_64-unknown-freebsd
#RUST_TRIPLES += x86_64-unknown-linux-musl
#RUST_TRIPLES += x86_64-unknown-netbsd
#RUST_TRIPLES += riscv64gc-unknown-linux-gnu
#RUST_TRIPLES += s390x-unknown-linux-gnu

ALL_BINARIES = $(foreach app,$(ALL_APPS),$(foreach triple,$(RUST_TRIPLES),$(foreach buildtype,$(BUILD_TYPES),target/$(triple)/$(buildtype)/$(app)$(if $(findstring windows_,$(platform)),.exe,))))
ALL_RUST_TOOLCHAIN = $(foreach triple,$(RUST_TRIPLES),rust-toolchain-$(triple))
# https://hub.docker.com/_/microsoft-windows-nanoserver
WINDOWS_VERSIONS = 1809 20H2 ltsc2022
BUILDX_BUILDER = buildx-builder

all: $(ALL_BINARIES)
assets: $(ASSETS)
protos: $(PROTOS)

# https://github.com/docker-library/official-images#architectures-other-than-amd64
images: DOCKER_PUSH = --push
images: linux-images windows-images
	-$(DOCKER) manifest rm $(FILETOOL_IMAGE):$(TAG)

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

linux-image-filetool: target/x86_64-unknown-linux-gnu/release/filetool
	$(DOCKER) buildx build --builder $(BUILDX_BUILDER) --platform linux/amd64 --build-arg BINARY_PATH=$< -f Dockerfile -t $(FILETOOL_IMAGE):$(TAG)-$* . $(DOCKER_PUSH)

ALL_WINDOWS_IMAGES = $(foreach app,$(ALL_APPS),$(foreach winver,$(WINDOWS_VERSIONS),windows-image-$(app)-$(winver)))
windows-images: $(ALL_WINDOWS_IMAGES)

windows-image-filetool-%: build/bin/windows_amd64/filetool.exe ensure-builder
	$(DOCKER) buildx build --builder $(BUILDX_BUILDER) --platform windows/amd64 -f Dockerfile.windows --build-arg WINDOWS_VERSION=$* -t $(FILETOOL_IMAGE):$(TAG)-windows_amd64-$* . $(DOCKER_PUSH)

rust-toolchain: $(ALL_RUST_TOOLCHAIN)

rust-toolchain-%:
	echo "Toolchain: $*"
	rustup target add $*

target/%/debug/filetool:
	echo "Building: $*"
	cargo build --target $*

target/%/release/filetool:
	echo "Building: $*"
	cargo build --target $* --release

deps:
	cargo update

fmt: $(ASSETS)
	cargo fmt

lint: fmt

test: $(ASSETS)
	cargo test

bench: $(ASSETS)
	cargo bench

check: lint test bench

clean:
	cargo clean
	rm -rf target/
	rm -rf build/
	rm -f duplicates.html checksums.txt output.html rmlist.txt

presubmit: clean check all coverage.txt

run:
	cargo run -- duplicate --path=$(PWD) --verbose=true --output=report.html

.PHONY: all deps fmt lint test bench check clean presubmit
