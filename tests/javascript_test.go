package main_test

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/assert"
)

func (suite *RoadRunnerTestSuite) TestJavascriptIterative() {
	var counter = 0

	var assertionFunction = func(response RoadRunnerResponse, t *testing.T) {
		if response.TerminalType == "StandardOutput" {
			assert.Equal(t, fmt.Sprintf("%d", counter), response.PipeValue)
			counter += 1
		}

		if response.TerminalType == "EndOfOutput" {
			assert.Equal(t, "exit status: 0", response.Value.ExitStatus)
		}
	}

	testHeader(suite, []byte(`{
		"language": "javascript",
		"source": "const delay = ms => new Promise(res => setTimeout(res, ms));\nfor(let i = 0; i < 5; i++) { console.log(i); await delay(500) }",
		"nonce": "javascript-timed"
	}`), assertionFunction)
}

func (suite *RoadRunnerTestSuite) TestJavascriptIterativeImmediate() {
	var counter = 0

	var assertionFunction = func(response RoadRunnerResponse, t *testing.T) {
		if response.TerminalType == "StandardOutput" {
			assert.Equal(t, fmt.Sprintf("%d", counter), response.PipeValue)
			counter += 1
		}

		if response.TerminalType == "EndOfOutput" {
			assert.Equal(t, "exit status: 0", response.Value.ExitStatus)
		}
	}

	testHeader(suite, []byte(`{
		"language": "javascript",
		"source": "for(let i = 0; i < 5; i++) { console.log(i) }",
		"nonce": "javascript-iterative"
	}`), assertionFunction)
}
