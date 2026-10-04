package session

import "time"

type Option func(*reactorOptions)

type reactorOptions struct {
	displayPanesTimeout time.Duration
}

func defaultReactorOptions() reactorOptions {
	return reactorOptions{displayPanesTimeout: 2 * time.Second}
}

func WithDisplayPanesTimeout(timeout time.Duration) Option {
	return func(options *reactorOptions) {
		if timeout > 0 {
			options.displayPanesTimeout = timeout
		}
	}
}
