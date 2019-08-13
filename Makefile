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

all: cmd/unique/unique$(DOT_EXE) cmd/similar/similar$(DOT_EXE)

cmd/similar/similar$(DOT_EXE):
	$(GO_BUILD) -o cmd/similar/similar$(DOT_EXE) cmd/similar/similar.go

cmd/unique/unique$(DOT_EXE):
	$(GO_BUILD) -o cmd/unique/unique$(DOT_EXE) cmd/unique/unique.go

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
