// Copyright 2022 Jeremy Edwards
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
	assert.True(p.CoarseHashing)
}
