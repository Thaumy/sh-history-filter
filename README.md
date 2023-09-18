# Sh history filter

Filter your shell history.

## Usage

```bash
sh-history-filter --shell-type bash --history-text "$(cat .bash_history)"
```

Configuration will be generated automatically in `~/shf.toml`

## Build

```bash
git clone --depth 1 https://github.com/Thaumy/sh-history-filter.git
cd sh-history-filter
cargo build -r
```

## Install over Nix

1. [Enable NUR](https://github.com/nix-community/NUR#installation)

2. Edit `configuration.nix` ：

```nix
environment.systemPackages = with pkgs; [
  nur.repos.thaumy.sh-history-filter
];
```

