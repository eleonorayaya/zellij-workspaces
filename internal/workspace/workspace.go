package workspace

type Workspace struct {
	ID        string `json:"id"`
	Name      string `json:"name"`
	Path      string `json:"path"`
	IsGitRepo bool   `json:"is_git_repo"`
}
