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

package unique

import (
	"fmt"
	"testing"
	"time"

	"github.com/jeremyje/filetools/testdata"
	"go.uber.org/zap/zaptest"
)

func TestUnique(t *testing.T) {
	var testCases = []struct {
		params *Params
	}{
		{&Params{Paths: []string{"."}, StatusFrequency: time.Second}},
		{&Params{Paths: []string{testdata.GetDirectory(t)}, StatusFrequency: time.Second}},
	}
	for _, tc := range testCases {
		tc := tc
		t.Run(fmt.Sprintf("Run(%+v)", tc.params), func(t *testing.T) {
			r, err := run(zaptest.NewLogger(t), tc.params)

			if err != nil {
				t.Error(err)
			}
			if r == nil {
				t.Error("result is nil")
			}
		})
	}
}
