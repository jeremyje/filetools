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

GO = go
GO_BUILD = CGO_ENABLED=0 go build -a -installsuffix cgo

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
	rm -f cmd/similar/similar$(DOT_EXE)
	rm -f cmd/unique/unique$(DOT_EXE)

presubmit: clean check all

.PHONY: all fmt vet lint test bench check clean presubmit
