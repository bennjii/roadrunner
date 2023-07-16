package main_test

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/assert"
)

func (suite *RoadRunnerTestSuite) TestPythonIO() {
	var assertionFunction = func(response RoadRunnerResponse, t *testing.T) {
		if response.TerminalType == "StandardOutput" {
			assert.Equal(t, "hello world!!! [or] This is another", response.PipeValue)
		}

		if response.TerminalType == "EndOfOutput" {
			assert.Equal(t, "exit status: 0", response.Value.ExitStatus)
		}
	}
	testHeader(suite, `{
		"language": "python",
		"source": "input_value = input()\ninput_value2 = input()\nprint(input_value + ' [or] ' + input_value2)",
		"nonce": "python-io",
		"standard_input": "hello world!!!\nThis is another"
	}`, assertionFunction)
}

func (suite *RoadRunnerTestSuite) TestPythonIterative() {
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

	testHeader(suite, `{
		"language": "python",
		"source": "import time\nfor i in range(5):\n\tprint(i)\n\ttime.sleep(0.5)",
		"nonce": "python-timed"
	}`, assertionFunction)
}

func (suite *RoadRunnerTestSuite) TestPythonIterativeImmediate() {
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

	testHeader(suite, `{
		"language": "python",
		"source": "import time\nfor i in range(5):\n\tprint(i)\n\ttime.sleep(0.5)",
		"nonce": "python-iterative"
	}`, assertionFunction)
}
