# 🎨 Zeteo CLI - Enhanced REPL Showcase

This document showcases the beautifully redesigned REPL mode that is now the flagship feature of Zeteo CLI.

## 🌟 Welcome Screen

When you start Zeteo, you're greeted with a stunning welcome screen:

```
  ╔═══════════════════════════════════════════════════════════════╗
  ║                                                               ║
  ║   ███████╗███████╗████████╗███████╗ ██████╗                 ║
  ║   ╚══███╔╝██╔════╝╚══██╔══╝██╔════╝██╔═══██╗                ║
  ║     ███╔╝ █████╗     ██║   █████╗  ██║   ██║                ║
  ║    ███╔╝  ██╔══╝     ██║   ██╔══╝  ██║   ██║                ║
  ║   ███████╗███████╗   ██║   ███████╗╚██████╔╝                ║
  ║   ╚══════╝╚══════╝   ╚═╝   ╚══════╝ ╚═════╝                 ║
  ║                                                               ║
  ║        AI-Powered OTEL Log Explorer & Chat Assistant         ║
  ║                                                               ║
  ╚═══════════════════════════════════════════════════════════════╝

┌─ Provider: 🤖 openai
└─ Log Explorer: ✓ Connected

╭──────────── Available Commands ────────────╮
│                                            │
│  🚪 /exit, /quit, /q   → Exit the REPL
│  🗑️ /clear            → Clear conversation history
│  ❓ /help, /h         → Show detailed help
│  🔍 /logs <query>     → Search OTEL logs
│  📊 /stats            → Show session statistics
│  💾 /export [file]    → Export conversation
│  📜 /history          → Show conversation history
│                                            │
╰────────────────────────────────────────────╯

💡 Tip: Just type your message to start chatting!
   Press Ctrl+C or type /exit to quit.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

zeteo [0]> 
```

## 💬 Chat Interaction

Beautiful AI response formatting with borders and timing:

```
zeteo [0]> What is OpenTelemetry?

💭 Thinking...

┌─ AI Response ─────────────────────────────────────
OpenTelemetry is an observability framework for 
cloud-native software that provides a standardized 
way to collect and export telemetry data including:

- Distributed traces
- Metrics
- Logs

It's vendor-neutral and supports multiple languages
and platforms, making it ideal for modern microservices
architectures.
└───────────────────────────────────────────────────

⏱  Response time: 1.23s

zeteo [1]> 
```

## 📊 Session Statistics

View comprehensive session metrics with the `/stats` command:

```
zeteo [5]> /stats

╔══════════════════════════════════════════════════╗
║          Session Statistics                      ║
╚══════════════════════════════════════════════════╝

  💬 Total messages exchanged:     5
  📝 Messages in history:          10
  ⏱  Session duration:             0h 5m 32s
  🤖 AI Provider:                  openai
  🔍 Log Explorer:                 Connected ✓

  📊 Average time per exchange: 1.4s
```

## 📜 Conversation History

Beautiful history display with role indicators:

```
zeteo [3]> /history

╭────────── Conversation History ──────────╮
│                                          │
│ 👤 [1] You:
│   What is OpenTelemetry?
│                                          │
│ 🤖 [1] AI:
│   OpenTelemetry is an observability framework for c...
│                                          │
│ 👤 [2] You:
│   How do I instrument a Node.js application?
│                                          │
│ 🤖 [2] AI:
│   To instrument a Node.js application with OpenTele...
│                                          │
│ 👤 [3] You:
│   Can you show me a code example?
│                                          │
│ 🤖 [3] AI:
│   Here's a complete example with all the required p...
│                                          │
╰──────────────────────────────────────────╯

💡 Tip: Use /export to save full conversation
```

## 🔍 Log Search

Search OTEL logs directly from the REPL:

```
zeteo [2]> /logs error

🔍 Searching logs for: error

[2024-01-15T10:23:45Z] ERROR Database connection timeout
  Service: api-gateway
  Level: ERROR
  Message: Failed to connect to PostgreSQL after 3 attempts
  
[2024-01-15T10:24:12Z] ERROR Authentication failed
  Service: auth-service
  Level: ERROR
  Message: Invalid JWT token signature
```

## 🔄 Provider Information

Check your current AI provider:

```
zeteo [2]> /provider

╭──────── Provider Information ────────╮
│                                      │
│  🤖 Name: openai                     │
│  📋 Model: GPT-4o / GPT-4            │
│                                      │
╰──────────────────────────────────────╯
```

## 💾 Export Conversation

Save your conversation with success feedback:

```
zeteo [5]> /export debugging-session.json

✓ Conversation exported to: debugging-session.json
```

## ❓ Enhanced Help

Comprehensive help with tips and tricks:

```
zeteo [1]> /help

╔══════════════════════════════════════════════════════════════╗
║                  Zeteo REPL Commands                         ║
╚══════════════════════════════════════════════════════════════╝

  🚪 /exit, /quit, /q      Exit the REPL and end session
  🗑️ /clear               Clear conversation history to start fresh
  ❓ /help, /h            Show this detailed help message
  🔍 /logs <query>        Search OTEL logs (e.g., /logs error)
  🔄 /provider            Show current AI provider info
  📊 /stats               Display session statistics
  💾 /export [file]       Export conversation (json/csv)
  📜 /history             Show conversation history summary

╭──────────────────────────────────────────────────────────────╮
│  💡 Tips & Tricks                                            │
├──────────────────────────────────────────────────────────────┤
│  • Just type your message to chat with AI                   │
│  • Use multi-line input with Shift+Enter (if supported)     │
│  • Export conversations for sharing with your team          │
│  • Check /stats to see your session activity                │
╰──────────────────────────────────────────────────────────────╯
```

## 👋 Goodbye Message

Friendly exit with session summary:

```
zeteo [5]> /exit

╔═══════════════════════════════════════════════════════════╗
║                 Thank You for Using Zeteo!               ║
╚═══════════════════════════════════════════════════════════╝

📊 Session Summary:
   5 messages exchanged in 8 minutes

💡 Tip: Don't forget to export your conversation with /export

👋 Goodbye!
```

## 🌈 Color Scheme

The REPL uses intelligent color coding throughout:

- **🟢 Green**: Success messages, AI responses, confirmations
- **🟡 Yellow**: Warnings, tips, informational alerts
- **🔴 Red**: Errors, critical issues
- **🔵 Cyan**: Commands, prompts, section headers
- **🟣 Magenta**: Statistics, highlights, special info
- **⚫ Dimmed**: Timestamps, less important details

## ✨ Key Features

### Visual Design
- ✅ Beautiful ASCII art banner
- ✅ Provider-specific emoji icons
- ✅ Professional borders and dividers
- ✅ Rich color scheme
- ✅ Clean, modern layout

### User Experience
- ✅ Response time tracking
- ✅ Session statistics
- ✅ Message counter in prompt
- ✅ Helpful tips throughout
- ✅ Smart error handling

### Functionality
- ✅ Conversation history
- ✅ Export to JSON/CSV
- ✅ Log search integration
- ✅ Multi-provider support
- ✅ Context-aware AI chat

---

## 🚀 Try It Yourself!

```bash
# Install Zeteo
cargo build --release

# Set up your API key
export OPENAI_API_KEY="your-key-here"

# Launch the beautiful REPL
./target/release/zeteo

# Or with a specific provider
./target/release/zeteo --provider google
```

For more information, see the [REPL Guide](examples/REPL_GUIDE.md).
