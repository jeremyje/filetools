package internal

import (
	"testing"
)

func TestNormalize(t *testing.T) {
	var tc = []struct {
		in         string
		out        string
		clearToken []string
	}{
		{"/Video/1.mp4", "1", []string{}},
		{"/Video/1-ok.mp4", "1ok", []string{"-"}},
		{"/Video/1-ok.mp4", "1", []string{"-ok"}},
		{"/Video/1 0 1.mp4", "101", []string{""}},
		{"/Video/101.mp4", "101", []string{}},
	}
	for _, tt := range tc {
		p := normalize(tt.in, tt.clearToken)
		if p != tt.out {
			t.Errorf("normalize(%s) > %s, expected %s", tt.in, p, tt.out)
		}
	}
}
