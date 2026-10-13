package main

import (
	"bufio"
	"fmt"
	"os"
	"os/signal"
	"syscall"
)

func main() {
	signals := make(chan os.Signal, 1)
	signal.Notify(signals, os.Interrupt, syscall.SIGTERM)
	defer signal.Stop(signals)

	fmt.Println("TERMAN_SCREEN_E2E_READY")
	input := make(chan string, 1)
	go readInput(input)
	for {
		select {
		case <-signals:
			fmt.Println("TERMAN_SCREEN_E2E_STOP")
			return
		case data, open := <-input:
			if !open {
				return
			}
			fmt.Printf("ECHO:%s", data)
		}
	}
}

func readInput(output chan<- string) {
	defer close(output)
	reader := bufio.NewReader(os.Stdin)
	for {
		data, err := reader.ReadString('\n')
		if data != "" {
			output <- data
		}
		if err != nil {
			return
		}
	}
}
