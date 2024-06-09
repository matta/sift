package main

import (
	"fmt"
	"log"
	"log/slog"
	"os"
	"path/filepath"

	"github.com/charmbracelet/bubbles/help"
	"github.com/charmbracelet/bubbles/key"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

type teaModel struct {
	help         *help.Model
	windowWidth  int
	windowHeight int
}

// Init implements tea.Model.
func (outer teaModel) Init() tea.Cmd {
	slog.Debug("teaModel.Init()")
	return nil
}

func newModel() teaModel {
	help := help.New()
	return teaModel{
		help: &help,
	}
}

// keyMap holds a set of keybindings. To work for help it must satisfy
// key.Map. It could also very easily be a map[string]key.Binding.
type keyMap struct {
	Help key.Binding
	Quit key.Binding
}

// ShortHelp returns keybindings to be shown in the mini help view. It's part
// of the key.Map interface.
func (k keyMap) ShortHelp() []key.Binding {
	return []key.Binding{k.Help, k.Quit}
}

// FullHelp returns keybindings for the expanded help view. It's part of the
// key.Map interface.
func (k keyMap) FullHelp() [][]key.Binding {
	return [][]key.Binding{
		{k.Help, k.Quit},
	}
}

var keys = keyMap{
	Help: key.NewBinding(
		key.WithKeys("?"),
		key.WithHelp("?", "toggle help"),
	),
	Quit: key.NewBinding(
		key.WithKeys("q", "esc", "ctrl+c"),
		key.WithHelp("q", "quit"),
	),
}

func (m teaModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.WindowSizeMsg:
		// If we set a width on our sub-models, so they can respond as needed.
		m.help.Width = msg.Width
		m.windowWidth = msg.Width
		m.windowHeight = msg.Height
	case tea.KeyMsg:
		cmd := m.handleKeyMsg(msg)
		return m, cmd
	}

	return m, nil
}

func (m teaModel) handleKeyMsg(msg tea.KeyMsg) tea.Cmd {
	switch {
	case key.Matches(msg, keys.Help):
		m.help.ShowAll = !m.help.ShowAll
	case key.Matches(msg, keys.Quit):
		return tea.Quit
	}

	return nil
}

func (m teaModel) View() string {
	out := fmt.Sprint("The screen has ", m.windowWidth, " columns and ", m.windowHeight, " rows\n")
	if m.windowWidth > 0 {
		out += m.help.View(keys)

		style := lipgloss.NewStyle().BorderStyle(lipgloss.NormalBorder())
		out = style.Render(out)
	}

	return out
}

func UserHomeDir() string {
	usr, err := os.UserHomeDir()
	if err != nil {
		log.Fatal(err)
	}

	return usr
}

func UserDataFile() string {
	return filepath.Join(UserHomeDir(), ".sift.yaml")
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

	teaModel := newModel()

	program := tea.NewProgram(teaModel, tea.WithAltScreen())
	_, err := program.Run()
	if err != nil {
		slog.Error("Error running program: %v", err)
	}

	slog.Debug("program exiting")
}
