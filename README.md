<h1 align="center">cmdflow🌈</h1>

<p align="center">
  <img src="https://img.shields.io/badge/build-passing-brightgreen?style=for-the-badge&logo=github-actions" alt="Build">
  <img src="https://img.shields.io/github/repo-size/voideez/cmdflow?style=for-the-badge&logo=gitlab" alt="Repo Size">
  <img src="https://img.shields.io/github/last-commit/voideez/cmdflow?style=for-the-badge&logo=git" alt="Last Commit">
  <img src="https://img.shields.io/badge/Language-Rust%201.97.0-white?style=for-the-badge&logo=rust" alt="Language Rust">
  <img src="https://img.shields.io/badge/Version-3.0.2-blue?style=for-the-badge&logo=github" alt="Version">
  <img src="https://img.shields.io/badge/OS-Unix--like-orange?style=for-the-badge&logo=linux" alt="OS">
  <img src="https://img.shields.io/badge/License-MIT-lightgrey?style=for-the-badge&logo=opensourcehardware" alt="License">
</p>

## [Russian README.md](README_ru.md)

cmdflow is a CLI utility for Linux that counts your Fish/Bash/Zsh commands, generates a Top-N list, and displays it beautifully right in your terminal.

- Automatic command caching on every run
- Support for both legacy and modern commands
- Top-N leaderboard with medals and colored progress bars
- Perfect for analyzing your terminal activity

---

### Features

- Counts all entered Fish, Bash, and Zsh commands, including duplicates
- Extracts only the base command (e.g., cargo build to cargo)
- Renders commands in a vibrant rainbow gradient alley
- Automatically updates logs on every execution

**Example output for cmdflow --working 50:**

---

⚠️ Important Note on Command Duplicates

By default, most shells (especially Fish and Bash) operate in ignoredups mode. This means if you type fastfetch 5 times in a row, only a single invocation gets recorded in your history.

To ensure your cmdflow charts reflect a 100% accurate picture of your actual activity, run the built-in configuration fixer:

```bash
cmdflow --fix-history

```

This command will safely append the necessary hooks to your ~/.bashrc and ~/.config/fish/config.fish files, forcing them to save absolutely every keystroke. Good luck.

---

### Installation

**Via AUR:**

If you use yay:

```bash
yay -S cmdflow

```

**From Source (GitHub):**

```bash
git clone https://github.com/voideez/cmdflow.git
cd cmdflow
cargo build --release
mkdir -p ~/.local/bin
ln -sf "$(pwd)/target/release/cmdflow" ~/.local/bin/cmdflow

```

---

Now the cmdflow command is globally available in your terminal:

```bash
cmdflow          # top 10 (fish + bash + zsh)
--fish           # only fish
--bash           # only bash
--zsh            # only zsh
cmdflow 15       # top 15 (fish + bash + zsh)
--fish 20        # top 20 (only fish)
--working        # only working commands
--broken         # only non-working commands
--fix-history    # history fix

```

---

### Requirements

* Cargo
* Fish shell
* Bash shell
* Zsh shell

---

### Development

Clone the repository:

```bash
git clone https://github.com/voideez/cmdflow.git
cd cmdflow

```

Build and run in development mode:

```bash
cargo build
cargo run

```

To update the local binary after making changes:

```bash
cargo build
ln -sf "$(pwd)/target/debug/cmdflow" ~/.local/bin/cmdflow
cmdflow

```
