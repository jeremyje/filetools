package main

import (
	"fmt"
	"github.com/stretchr/testify/assert"
	"testing"
)

func TestFromFlags(t *testing.T) {
	assert := assert.New(t)
	p := fromFlags()
	assert.Empty(p.Path)
	fmt.Printf("%v .. len= %d\n", p.ClearTokens, len(p.ClearTokens))
	assert.Empty(p.ClearTokens)
}
