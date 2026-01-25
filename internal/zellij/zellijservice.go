package zellij

import "context"

type ZellijSession struct {
	name string
}

type ZellijService struct {
	sessions []ZellijSession
}

func NewZellijService() *ZellijService {
	z := &ZellijService{}

	return z
}

func (z *ZellijService) OnSessionUpdate(ctx context.Context) error {

	return nil
}
