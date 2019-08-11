package main

import (
	"github.com/stretchr/testify/assert"
	"testing"
)

func TestFromFlags(t *testing.T) {
	assert := assert.New(t)
	p := fromFlags()
	assert.Empty(p.Paths)
	assert.Zero(p.MinSize)
	assert.Empty(p.DeletePaths)
	assert.True(p.DryRun)
	assert.Empty(p.ReportFile)
	assert.False(p.Verbose)
	assert.True(p.Overwrite)
	assert.Equal(p.HashFunction, "crc64")
}
