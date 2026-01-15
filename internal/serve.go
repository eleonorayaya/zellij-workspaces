package internal

import (
	"fmt"
	"net/http"

	"github.com/spf13/cobra"
)

var ServeCmd = &cobra.Command{
	Use:   "serve",
	Short: "",
	Run:   serve,
}

func serve(cmd *cobra.Command, args []string) {
	mux := http.NewServeMux()

	mux.HandleFunc("GET /", func(w http.ResponseWriter, r *http.Request) {
		fmt.Fprint(w, "got path\n")
	})

	http.ListenAndServe("localhost:8080", mux)
}
