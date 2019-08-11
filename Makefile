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
	go fmt ./...

vet:
	$(GO) vet ./...

lint: fmt vet

test:
	$(GO) test ./... -race -cover

bench:
	$(GO) test ./... -bench=.

check: lint test bench

clean:
	rm -f cmd/similar/similar$(DOT_EXE)
	rm -f cmd/unique$/unique$(DOT_EXE)

.PHONY: all fmt vet lint test check clean
