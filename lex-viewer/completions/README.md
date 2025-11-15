# Shell Completions for lexv

This directory contains shell completion scripts for `lexv` that enable path auto-completion.

## Installation

### Bash

Add the following to your `~/.bashrc` or `~/.bash_profile`:

```bash
source /path/to/lex/lex-viewer/completions/lexv.bash
```

Or copy the file to your bash completions directory:

```bash
# On Linux
sudo cp lexv.bash /etc/bash_completion.d/

# On macOS (with Homebrew)
cp lexv.bash $(brew --prefix)/etc/bash_completion.d/
```

### Zsh

Copy the completion file to a directory in your `$fpath`:

```zsh
# Find your zsh completions directory
echo $fpath

# Copy the file (example for macOS with Homebrew)
cp _lexv /usr/local/share/zsh/site-functions/

# Or add to your custom completions directory
mkdir -p ~/.zsh/completions
cp _lexv ~/.zsh/completions/
# Add to ~/.zshrc:
fpath=(~/.zsh/completions $fpath)
autoload -Uz compinit && compinit
```

### Fish

Copy the completion file to your fish completions directory:

```fish
cp lexv.fish ~/.config/fish/completions/
```

## Usage

After installation, restart your shell or source your shell configuration file. Then:

```bash
lexv <TAB>
```

Will auto-complete file paths.

## Features

- Auto-completes file paths for the document argument
- Supports `-h`, `--help`, `-V`, `--version` flags
- Works with relative and absolute paths
