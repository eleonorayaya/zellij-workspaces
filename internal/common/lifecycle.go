package common

import "context"

type Module interface {
	OnAppStart(ctx context.Context) error
	OnAppEnd(ctx context.Context) error
}
