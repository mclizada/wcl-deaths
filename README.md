# wcl-deaths

Analyzes WarcraftLogs reports to surface bad deaths on a specific encounter — tracking deaths to avoidable abilities and ranking players by frequency and death order.

## Setup

### 1. Credentials

Create a `.env` file in the project root with your WarcraftLogs API credentials:

```
WCL_CLIENT_ID=your_client_id
WCL_CLIENT_SECRET=your_client_secret
```

Credentials can be obtained from the [WarcraftLogs API client manager](https://www.warcraftlogs.com/api/clients/).

### 2. Config

Edit `config.toml` to define encounters and their bad ability IDs:

```toml
[[encounter]]
id = 3134
name = "Nexus-King Salhadaar"
bad_abilities = [
    1224812,  # Vanquish
    1224814,  # Vanquish (variant)
    1233702,  # Vanquish (killing blow)
    1227472,  # Besiege (actual damage)
]
```

Multiple `[[encounter]]` blocks are supported.

### 3. Build

```
cargo build --release
```

## Usage

```
wcl-deaths -r <CODE> [-r <CODE> ...] [-e <ENCOUNTER_ID>]
```

### Options

| Flag | Short | Description | Default |
|------|-------|-------------|---------|
| `--reports` | `-r` | Report code(s) to analyze. Repeat for multiple. | *(required)* |
| `--encounter` | `-e` | Encounter ID (must exist in config.toml) | `3134` |
| `--help` | `-h` | Print help | |

### Examples

Single report:
```
cargo run -- -r AbVphwHqgLJ7ZQ3Y
```

Multiple reports aggregated together:
```
cargo run -- -r AbVphwHqgLJ7ZQ3Y -r GbkAZP4Hwvn68yfL
```

Different encounter:
```
cargo run -- -r AbVphwHqgLJ7ZQ3Y --encounter 3135
```

- **bad deaths** — deaths where the killing blow was a configured bad ability
- **avg death order** — average position when dying across *all* deaths (not just bad ones)
- **top-3 deaths** — how many times the player was among the first 3 to die in a pull
