package main_test

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/assert"
)

func (suite *RoadRunnerTestSuite) TestGoIterative() {
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
		"language": "go",
		"source": "package main\n\nimport (\n\t\"fmt\"\n\t\"time\"\n)\n\nfunc main() {\n\tfor i := 0; i <= 4; i++ {\n\t\tfmt.Println(i)\n\t\ttime.Sleep(500 * time.Millisecond)\n\t}\n}",
		"nonce": "go-timed"
	}`), assertionFunction)
}

func (suite *RoadRunnerTestSuite) TestGoIterativeImmediate() {
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
		"language": "go",
		"source": "package main\n\nimport \"fmt\"\n\nfunc main() {\n\tfor i := 0; i <= 4; i++ {\n\t\tfmt.Println(i)\n\t}\n}",
		"nonce": "go-iterative"
	}`), assertionFunction)
}
