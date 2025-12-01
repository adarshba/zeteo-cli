# VHS Demo Tape for Zeteo
# Run with: vhs demo.tape
# Install vhs: https://github.com/charmbracelet/vhs

Output assets/demo.gif

Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set Padding 20

# Start zeteo
Type "zeteo"
Enter
Sleep 2s

# Show the welcome screen
Sleep 2s

# Ask a question about logs
Type "Show me the last 5 errors from the logs"
Sleep 500ms
Enter
Sleep 4s

# Show slash commands
Type "/"
Sleep 1s

# Navigate through commands
Down
Sleep 500ms
Down
Sleep 500ms
Up
Sleep 500ms

# Press Escape to close modal
Escape
Sleep 500ms

# Clear the input
Ctrl+u
Sleep 500ms

# Ask another question
Type "What services had issues in the last hour?"
Sleep 500ms
Enter
Sleep 4s

# Use slash command to show help
Type "/help"
Enter
Sleep 2s

# Scroll up to see history
Up
Sleep 500ms
Up
Sleep 500ms
Down
Sleep 500ms

# Exit
Type "/quit"
Enter
Sleep 1s
