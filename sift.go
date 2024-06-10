package main

import (
	"fmt"
	"log"
	"log/slog"
	"os"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
	"github.com/davecgh/go-spew/spew"
)

type model struct {
	width  int
	height int
}

// Init implements tea.Model.
func (outer model) Init() tea.Cmd {
	slog.Debug("teaModel.Init()")
	return nil
}

func (m model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	prefix := spew.Sprintf("Update(%#v)", msg)
	slog.Debug(fmt.Sprintf("%s ENTER", prefix))
	defer slog.Debug(fmt.Sprintf("%s LEAVE", prefix))

	switch msg := msg.(type) {
	case tea.WindowSizeMsg:
		m.width = msg.Width
		m.height = msg.Height
	case tea.KeyMsg:
		return m, tea.Quit
	}

	return m, nil
}

func (m model) View() string {
	slog.Debug("View() ENTER")
	defer slog.Debug("View() LEAVE")

	out := fmt.Sprint("The screen has ", m.width,
		" columns and ", m.height, " rows\n")

	if m.width > 0 {
		style := lipgloss.NewStyle().Foreground(lipgloss.AdaptiveColor{
			Light: "#DDDADA",
			Dark:  "#9C9C00",
		})
		out = style.Render(out)
	}

	return out
}

func setUpLogging() *os.File {
	logfilePath := os.Getenv("SIFT_LOGFILE")
	if logfilePath != "" {
		file, err := tea.LogToFileWith(logfilePath, "sift", log.Default())
		if err != nil {
			fmt.Printf("Error logging to file: %v\n", err)
			os.Exit(1)
		}

		log.Default().SetFlags(log.LstdFlags | log.Lmicroseconds | log.Llongfile)
		slog.SetLogLoggerLevel(slog.LevelDebug)

		return file
	}

	return nil
}

func main() {
	logFile := setUpLogging()
	defer func() {
		if logFile != nil {
			_ = logFile.Close()
		}
	}()
	slog.Info("program started")

	program := tea.NewProgram(model{}, tea.WithAltScreen())
	_, err := program.Run()
	if err != nil {
		slog.Error("Error running program: %v", err)
	}

	slog.Debug("program exiting")
}
