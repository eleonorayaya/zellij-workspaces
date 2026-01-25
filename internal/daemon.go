package internal

import (
	"os"
	"os/signal"
	"syscall"

	"github.com/eleonorayaya/utena/internal/zellij"
	"github.com/spf13/cobra"
)

var DaemonCommand = &cobra.Command{
	Use:   "daemon",
	Short: "",
	Run:   startDaemon,
}

func startDaemon(cmd *cobra.Command, args []string) {
	ctx, stop := signal.NotifyContext(cmd.Context(), os.Interrupt, syscall.SIGTERM)
	defer stop()

	workspaceMgr := NewWorkspaceManager()

	zellijSvc := zellij.NewZellijService()
	go serveAPI(ctx, workspaceMgr, zellijSvc)

	<-ctx.Done()
}
