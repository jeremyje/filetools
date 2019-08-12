// Copyright 2019 Jeremy Edwards
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

package internal

import (
	"fmt"
	"github.com/stretchr/testify/assert"
	"testing"
)

func TestUnique(t *testing.T) {
	var testCases = []struct {
		params *UniqueParams
	}{
		{&UniqueParams{Paths: []string{"."}}},
	}
	for _, tc := range testCases {
		tc := tc
		t.Run(fmt.Sprintf("uniqueScan(%+v)", tc.params), func(t *testing.T) {
			assert := assert.New(t)
			uc, err := uniqueScan(tc.params)
			assert.Nil(err)
			assert.NotNil(uc)
		})
	}
}
