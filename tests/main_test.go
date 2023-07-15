// client.go
package main

import (
	"encoding/json"
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

func (suite *RoadRunnerTestSuite) TestPython() {
	// Channel for receiving the response
	responseCh := make(chan RoadRunnerResponse)
	t := suite.T()
	conn := suite.conn

	go func() {
		// Write the value to the websocket connection
		err := conn.WriteMessage(websocket.TextMessage, []byte(`{
			"language": "python",
			"source": "input_value = input()\ninput_value2 = input()\nprint(input_value + ' [or] ' + input_value2)",
			"nonce": "25",
			"standard_input": "hello world!!!\nThis is another"
		}`))

		assert.NoError(t, err)

		// Read the response from the websocket connection
		_, response, err := conn.ReadMessage()
		assert.NoError(t, err)

		var myResponse RoadRunnerResponse
		err = json.Unmarshal(response, &myResponse)
		assert.NoError(t, err)

		responseCh <- myResponse
	}()

	// Wait for the response or timeout
	select {
	case response := <-responseCh:
		// Process the response

		if response.TerminalType == "StandardOutput" {
			assert.Equal(t, "hello world!!! [or] This is another", response.PipeValue)
		}

		if response.TerminalType == "EndOfOutput" {
			assert.Equal(t, "exit status: 0", response.Value.ExitStatus)
		}
	case <-time.After(5 * time.Second):
		t.Error("Timeout occurred")
	}
}

func TestSuite(t *testing.T) {
	suite.Run(t, new(RoadRunnerTestSuite))
}
