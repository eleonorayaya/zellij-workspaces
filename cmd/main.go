package main

import (
	"fmt"
	"os"

	"github.com/eleonorayaya/utena/internal"
	"github.com/spf13/cobra"
)

var rootCmd = &cobra.Command{
	Use:   "utena",
	Short: "",
	Long:  ``,
}

func init() {
	rootCmd.AddCommand(internal.ServeCmd)
}

func main() {
	if err := rootCmd.Execute(); err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
}
