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
	"testing"
)

func TestWorkQueue(t *testing.T) {
	var testCases = []struct {
		name string
		in   [][]int
		want []int
	}{
		{
			name: "empty",
			in:   [][]int{},
			want: []int{},
		},
		{
			name: "one",
			in: [][]int{
				{1},
				{1},
			},
			want: []int{},
		},
		{
			name: "ten",
			in: [][]int{
				{1, 2, 3, 4, 5, 6},
				{1},
				{7, 8, 9, 10},
				{2, 3, 4, 5, 6, 7, 8, 9, 10},
				{1},
				{1},
			},
			want: []int{},
		},
	}
	for _, tc := range testCases {
		tc := tc
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()

			q := newWorkQueue[int]()
			for idx, row := range tc.in {
				if idx%2 == 0 {
					for _, item := range row {
						q.Submit(item)
					}
				} else {
					for _, want := range row {
						got, ok := q.Consume()
						if !ok {
							t.Errorf("queue exhausted")
						}
						if want != got {
							t.Errorf("got: %d, want: %d", got, want)
						}
					}
				}
			}
		})
	}
}

func TestWorkQueueEmpty(t *testing.T) {
	q := newWorkQueue[int]()
	q.Submit(1)
	got, ok := q.Consume()
	if got != 1 {
		t.Errorf("got: %d, want: 1", got)
	}
	if !ok {
		t.Error("queue exhausted prematurely")
	}
	got, ok = q.Consume()
	if got != 0 {
		t.Errorf("got: %d, want: 0", got)
	}
	if ok {
		t.Error("still has content")
	}
}
