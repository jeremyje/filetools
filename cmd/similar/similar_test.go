package main

import (
	"github.com/stretchr/testify/assert"
	"testing"
)

func TestFromFlags(t *testing.T) {
	assert := assert.New(t)
	p := fromFlags()
	assert.Empty(p.Paths)
	assert.Empty(p.ClearTokens)
}
