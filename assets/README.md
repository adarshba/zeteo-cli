# Assets

This directory contains media assets for the Zeteo project.

## Demo GIF

To generate the demo GIF, you'll need [vhs](https://github.com/charmbracelet/vhs):

```bash
# Install vhs (macOS)
brew install vhs

# Install vhs (Linux)
# See: https://github.com/charmbracelet/vhs#installation

# Generate the demo
vhs demo.tape
```

This will create `assets/demo.gif` from the `demo.tape` script.

## Alternative Recording Methods

- **asciinema**: `asciinema rec demo.cast` then convert with `agg`
- **ttygif**: Record with `ttyrec` then convert with `ttygif`
- **terminalizer**: `npm install -g terminalizer && terminalizer record demo`
