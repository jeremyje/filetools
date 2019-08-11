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
