// client.go
package main_test

import (
	"encoding/json"
	"fmt"
	"log"
	"os"
	"os/signal"
	"testing"
	"time"

	"github.com/gorilla/websocket"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/suite"
)

type RoadRunnerTestSuite struct {
	suite.Suite
	conn *websocket.Conn
}

type RoadRunnerTermination struct {
	ExitStatus string      `json:"exit_status"`
	Duration   json.Number `json:"duration"`
}

type RoadRunnerResponse struct {
	TerminalType string                `json:"terminal_type"`
	Value        RoadRunnerTermination `json:"value"`
	PipeValue    string                `json:"pipe_value"`
	Nonce        string                `json:"nonce"`
	Timestamp    string                `json:"timestamp"`
}

var done chan interface{}
var interrupt chan os.Signal

func (suite *RoadRunnerTestSuite) SetupTest() {
	suite.T().Parallel()

	done = make(chan interface{})    // Channel to indicate that the receiverHandler is done
	interrupt = make(chan os.Signal) // Channel to listen for interrupt signal to terminate gracefully

	signal.Notify(interrupt, os.Interrupt) // Notify the interrupt channel for SIGINT

	socketUrl := "ws://localhost:443" + "/ws"
	conn, _, err := websocket.DefaultDialer.Dial(socketUrl, nil)
	if err != nil {
		log.Fatal("Error connecting to Websocket Server:", err)
	}

	suite.Require().NoError(err)
	suite.conn = conn
}

func (suite *RoadRunnerTestSuite) TearDownTest() {
	conn := suite.conn
	// Terminate gracefully...
	log.Println("Received TESTEND interrupt signal. Closing all pending connections")

	// Close our websocket connection
	err := conn.WriteMessage(websocket.CloseMessage, websocket.FormatCloseMessage(websocket.CloseNormalClosure, ""))
	if err != nil {
		log.Println("Error during closing websocket:", err)
		return
	}

	conn.Close()

	select {
	case <-done:
		log.Println("Receiver Channel Closed! Exiting....")
	case <-time.After(time.Duration(1) * time.Second):
		log.Println("Timeout in closing receiving channel. Exiting....")
	}
}

func testHeader(suite *RoadRunnerTestSuite, content string, assertion func(response RoadRunnerResponse, t *testing.T)) {
	// Channel for receiving the response
	responseCh := make(chan RoadRunnerResponse)

	doneCh := make(chan struct{})

	t := suite.T()
	conn := suite.conn

	fmt.Println("Starting Message", content)

	// 1s longer than standard timeout to check the internal timeout works.
	var timedOut = time.After(6 * time.Second)

	// Write the value to the websocket connection
	err := conn.WriteMessage(websocket.TextMessage, []byte(content))
	assert.NoError(t, err)

	go func() {
		defer close(doneCh)

		for {
			// Read the response from the websocket connection
			_, response, err := conn.ReadMessage()
			assert.NoError(t, err)

			var myResponse RoadRunnerResponse
			err = json.Unmarshal(response, &myResponse)
			assert.NoError(t, err)

			responseCh <- myResponse

			if myResponse.TerminalType == "EndOfOutput" {
				// Break out of the loop when the response is "EndOfOutput"
				return
			}
		}
	}()

	// Wait for the response or timeout
out:
	for {
		select {
		case response := <-responseCh:
			fmt.Println(response.Nonce, response.PipeValue, response.Value)
			// Process the response
			assertion(response, t)

			if response.TerminalType == "EndOfOutput" {
				break out
			}
		case <-timedOut:
			close(doneCh)

			fmt.Println("Timeout Occurred")
			t.Error("Timeout occurred")
			break out
		}
	}

	<-doneCh
}

func TestSuite(t *testing.T) {
	suite.Run(t, new(RoadRunnerTestSuite))
}
