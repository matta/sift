package main

import (
	"fmt"
	"github.com/charmbracelet/bubbles/help"
	"github.com/charmbracelet/bubbles/key"
	"github.com/charmbracelet/bubbles/textinput"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
	"github.com/charmbracelet/log"
	"github.com/davecgh/go-spew/spew"
	"github.com/ghodss/yaml"
	"github.com/matta/sift/internal/replicatedtodo"
	"os"
	"path/filepath"
	"slices"
	"strings"
)

type teaModel struct {
	wrapped *model
}

type model struct {
	keys            keyMap
	help            help.Model
	persisted       *replicatedtodo.Model
	cursorID        string
	textInput       textinput.Model
	acceptTextInput func(title string)
}

// Init implements tea.Model.
func (outer teaModel) Init() tea.Cmd {
	return nil
}

// Update implements tea.Model.
func (outer teaModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	log.Debugf("Update: %+v", spew.Sdump(msg))
	return outer, outer.wrapped.update(msg)
}

// View implements tea.Model.
func (outer teaModel) View() string {
	return outer.wrapped.view()
}

func newModel() *model {
	return &model{
		keys:            keys,
		help:            help.New(),
		persisted:       replicatedtodo.New(),
		textInput:       textinput.New(),
		acceptTextInput: nil,
		cursorID:        "",
	}
}

//goland:noinspection GoMixedReceiverTypes
func (m *model) addSampleItems() {
	m.newTodo("todo 1")
	m.newTodo("todo 2")
}

// keyMap holds a set of keybindings. To work for help it must satisfy
// key.Map. It could also very easily be a map[string]key.Binding.
type keyMap struct {
	Up     key.Binding
	Down   key.Binding
	Toggle key.Binding
	Add    key.Binding
	Edit   key.Binding
	Help   key.Binding
	Quit   key.Binding
	Cancel key.Binding
	Accept key.Binding
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
		{k.Up, k.Down, k.Toggle, k.Add}, // first column
		{k.Help, k.Quit},                // second column
	}
}

var keys = keyMap{
	Up: key.NewBinding(
		key.WithKeys("up", "k"),
		key.WithHelp("↑/k", "move up"),
	),
	Down: key.NewBinding(
		key.WithKeys("down", "j"),
		key.WithHelp("↓/j", "move down"),
	),
	Toggle: key.NewBinding(
		key.WithKeys("x"),
		key.WithHelp("x", "toggle item"),
	),
	Add: key.NewBinding(
		key.WithKeys("a"),
		key.WithHelp("a", "add item"),
	),
	Edit: key.NewBinding(
		key.WithKeys("e"),
		key.WithHelp("e", "edit item"),
	),
	Help: key.NewBinding(
		key.WithKeys("?"),
		key.WithHelp("?", "toggle help"),
	),
	Quit: key.NewBinding(
		key.WithKeys("q", "esc", "ctrl+c"),
		key.WithHelp("q", "quit"),
	),
	Cancel: key.NewBinding(key.WithKeys("esc")),
	Accept: key.NewBinding(
		key.WithKeys("enter")),
}

func (m *model) newTodo(title string) {
	m.persisted.NewTodo(title)
}

func (m *model) save() error {
	bytes, err := yaml.Marshal(&m.persisted)
	if err != nil {
		return fmt.Errorf("failed to marshal model: %w", err)
	}

	var PERM = 0600
	err = os.WriteFile(UserDataFile(), bytes, os.FileMode(PERM))

	if err != nil {
		return fmt.Errorf("failed to save model: %w", err)
	}

	return nil
}

func updateTextInput(input *textinput.Model, msg tea.Msg) tea.Cmd {
	temp, cmd := input.Update(msg)
	*input = temp

	return cmd
}

func (m *model) update(msg tea.Msg) tea.Cmd {
	switch msg := msg.(type) {
	case tea.WindowSizeMsg:
		// If we set a width on our sub-models, so they can respond as needed.
		m.help.Width = msg.Width
		m.textInput.Width = msg.Width
	case tea.KeyMsg:
		return m.handleKeyMsg(msg)
	}

	return nil
}

func (m *model) handleKeyMsg(msg tea.KeyMsg) tea.Cmd {
	switch {
	case m.textInput.Focused():
		return m.handleFocusedTextInput(msg)
	case key.Matches(msg, m.keys.Help):
		m.help.ShowAll = !m.help.ShowAll
	case key.Matches(msg, m.keys.Up):
		m.cursorUp()
	case key.Matches(msg, m.keys.Down):
		m.cursorDown()
	case key.Matches(msg, m.keys.Toggle):
		m.toggle()
	case key.Matches(msg, m.keys.Add):
		return m.add()
	case key.Matches(msg, m.keys.Edit):
		return m.edit()
	case key.Matches(msg, m.keys.Quit):
		return tea.Quit
	}

	return nil
}

func (m *model) toggle() {
	id := m.currentItem().ID
	m.persisted.ToggleDone(id)
	log.Debugf("persisted after toggling %s\n%s", id, m.persisted.DebugString())
}

func (m *model) add() tea.Cmd {
	m.acceptTextInput = func(title string) {
		m.newTodo(title)
		m.disableTextInput()
		m.acceptTextInput = nil
	}

	return m.textInput.Focus()
}

func (m *model) edit() tea.Cmd {
	m.acceptTextInput = func(title string) {
		if len(title) > 0 {
			m.persisted.SetTitle(m.currentItem().ID, title)
			m.disableTextInput()
		}

		m.acceptTextInput = nil
	}
	m.textInput.SetValue(m.currentItem().Title)

	return m.textInput.Focus()
}

func findID(items []replicatedtodo.Item, cursorID string) int {
	index := slices.IndexFunc(items, func(item replicatedtodo.Item) bool {
		return item.ID == cursorID
	})

	return index
}

func (m *model) currentItem() replicatedtodo.Item {
	items := m.loadItems()
	index := findID(items, m.cursorID)

	return items[index]
}

func (m *model) cursorDown() {
	items := m.loadItems()
	index := findID(items, m.cursorID)

	switch {
	case len(items) == 0:
		m.cursorID = ""
	case index >= 0 && index < len(items)-1:
		m.cursorID = items[index+1].ID
	default:
		m.cursorID = items[0].ID
	}
}

func (m *model) cursorUp() {
	items := m.loadItems()
	index := findID(items, m.cursorID)

	switch {
	case len(items) == 0:
		m.cursorID = ""
	case index > 0:
		m.cursorID = items[index-1].ID
	default:
		m.cursorID = items[len(items)-1].ID
	}
}

func (m *model) handleFocusedTextInput(msg tea.KeyMsg) tea.Cmd {
	switch {
	case key.Matches(msg, m.keys.Cancel):
		m.disableTextInput()
	case key.Matches(msg, m.keys.Accept):
		m.accept()
	default:
		return updateTextInput(&m.textInput, msg)
	}

	return nil
}

func (m *model) accept() {
	m.acceptTextInput(m.textInput.Value())
	m.acceptTextInput = nil
	m.disableTextInput()
}

func (m *model) disableTextInput() {
	m.textInput.Reset()
	m.textInput.Blur()
}

func (m *model) view() string {
	out := ""

	items := m.loadItems()

	for _, item := range items {
		cursor := " "
		if item.ID == m.cursorID {
			cursor = ">"
		}

		done := " "
		if item.State == "checked" {
			done = "x"
		}

		out += fmt.Sprintf("%s [%s] %s\n", cursor, done, item.Title)
	}

	if m.textInput.Focused() {
		out += m.textInput.View()
	} else {
		out += m.help.View(m.keys)
	}

	style := lipgloss.NewStyle().BorderStyle(lipgloss.NormalBorder())
	out = style.Render(out)

	return out
}

func (m *model) loadItems() []replicatedtodo.Item {
	items := m.persisted.GetAllItems()
	items = slices.DeleteFunc(items, func(item replicatedtodo.Item) bool {
		return item.State == "removed"
	})
	slices.SortFunc(items, func(i, j replicatedtodo.Item) int {
		return strings.Compare(i.ID, j.ID)
	})

	return items
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

func loadModel() *teaModel {
	model := loadPersistedModel()

	return &teaModel{model}
}

func loadPersistedModel() *model {
	model := newModel()

	bytes, err := os.ReadFile(UserDataFile())
	if err != nil {
		log.Printf("Failed to read model file: %v", err)
		model.addSampleItems()

		return model
	}

	var replicatedModel replicatedtodo.Model
	if err = yaml.Unmarshal(bytes, &replicatedModel); err != nil {
		log.Printf("Failed to unmarshal model file: %v", err)
		model.addSampleItems()

		return model
	}

	model.persisted = &replicatedModel

	return model
}

func setUpLogging() *os.File {
	logfilePath := os.Getenv("SIFT_LOGFILE")
	if logfilePath != "" {
		file, err := tea.LogToFileWith(logfilePath, "sift", log.Default())
		if err != nil {
			fmt.Printf("Error logging to file: %v\n", err)
			os.Exit(1)
		}

		log.SetLevel(log.DebugLevel)
		log.SetReportCaller(true)
		log.SetReportTimestamp(true)
		log.Debug("Debug logging enabled")

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

	teaModel := loadModel()

	program := tea.NewProgram(teaModel, tea.WithAltScreen())
	_, err := program.Run()

	if err != nil {
		log.Errorf("Error running program: %v", err)
	}

	if err := teaModel.wrapped.save(); err != nil {
		log.Errorf("Error saving: %v", err)
	}

	log.Debug("program exiting")
}