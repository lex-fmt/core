# Shell Completions for lex

This directory contains shell completion scripts for the `lex` CLI tool that enable path auto-completion for the file argument and format auto-completion for the transform argument.

## Installation

### Bash

Add the following to your `~/.bashrc` or `~/.bash_profile`:

```bash
source /path/to/lex/lex-cli/completions/lex.bash
```

Or copy the file to your bash completions directory:

```bash
# On Linux
sudo cp lex.bash /etc/bash_completion.d/

# On macOS (with Homebrew)
cp lex.bash $(brew --prefix)/etc/bash_completion.d/
```

### Zsh

Copy the completion file to a directory in your `$fpath`:

```zsh
# Find your zsh completions directory
echo $fpath

# Copy the file (example for macOS with Homebrew)
cp _lex /usr/local/share/zsh/site-functions/

# Or add to your custom completions directory
mkdir -p ~/.zsh/completions
cp _lex ~/.zsh/completions/
# Add to ~/.zshrc:
fpath=(~/.zsh/completions $fpath)
autoload -Uz compinit && compinit
```

### Fish

Copy the completion file to your fish completions directory:

```fish
cp lex.fish ~/.config/fish/completions/
```

## Usage

After installation, restart your shell or source your shell configuration file. Then:

```bash
# First argument: file path completion
lex <TAB>

# Second argument: transform format completion
lex myfile.txt <TAB>
```

## Features

- Auto-completes file paths for the first argument
- Auto-completes transform formats for the second argument:
  - `token-core-json`, `token-core-simple`, `token-core-pprint`
  - `token-simple`, `token-pprint` (aliases)
  - `token-line-json`, `token-line-simple`, `token-line-pprint`
  - `ir-json`
  - `ast-json`, `ast-tag`, `ast-treeviz`
- Supports flags: `-h`, `--help`, `-V`, `--version`, `--list-transforms`
- Works with relative and absolute paths
