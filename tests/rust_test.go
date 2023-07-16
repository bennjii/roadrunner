package main_test

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/assert"
)

func implContains(sl []string, name string) bool {
	// iterate over the array and compare given string to each element
	for _, value := range sl {
		if value == name {
			return true
		}
	}
	return false
}

func (suite *RoadRunnerTestSuite) TestRustIO() {
	outputs := []string{"A", "B"}

	var assertionFunction = func(response RoadRunnerResponse, t *testing.T) {
		if response.TerminalType == "StandardOutput" {
			assert.True(t, implContains(outputs, response.PipeValue))
		}

		if response.TerminalType == "EndOfOutput" {
			assert.Equal(t, "exit status: 0", response.Value.ExitStatus)
		}
	}
	testHeader(suite, `{
		"language": "rust",
		"source": "use std::io;\nfn main() {\n\tfor line in io::stdin().lines() {\n\t\tprint!(\"{}\", line.unwrap());\n\t\tbreak;\n\t}\n}",
		"nonce": "rust-io",
		"standard_input": "A\nB"
	}`, assertionFunction)
}

func (suite *RoadRunnerTestSuite) TestRustIterative() {
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
		"language": "rust",
		"source": "use std::time::Duration;\nuse std::thread;\nfn main() {\n\tfor i in 0..5 {\n\t\tprintln!(\"{}\", i);\n\t\tthread::sleep(Duration::from_millis(500))\n\t}\n}",
		"nonce": "rust-timed"
	}`, assertionFunction)
}

func (suite *RoadRunnerTestSuite) TestRustIterativeImmediate() {
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
		"language": "rust",
		"source": "use std::thread;\nfn main() {\n\tfor i in 0..5 {\n\t\tprintln!(\"{}\", i);\n\t}\n}",
		"nonce": "rust-iterative"
	}`, assertionFunction)
}
