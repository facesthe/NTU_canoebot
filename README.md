# NTU canoebot
A bot that automates daily tasks for NTU Canoe Sprint

---

## Prerequisites

- [Install rust](https://rustup.rs)

- [Install docker engine](https://docs.docker.com/engine/install/)

---

## Building
Before building, [configure settings first.](#configuring-settings)


### Dev
```sh
cargo run
```

### Deploy
```sh
# if you have make on your system
make    # build
make up # run

# if you don't have make on your system
docker build -t ntu_canoebot_cache -f docker/cache.Dockerfile .
docker compose build
docker compose up -d
```

## Configuring settings
Before building the bot, it needs to be set up with an API key and to point to all necessary Google resource IDs in order to function properly.

These keys are used to generate code at compile-time for the bot,
so that all configuration data is "baked" into the executable.

Create 2 copies of `botsettings.template.d.toml`:
- `botsettings.template.deploy.toml` (mandatory)
- `botsettings.template.debug.toml` (optional)

Fill in all keys except those marked optional

### Review
Altogether there should now be 4 'template' TOML files inside `.configs`:
- `botsettings.template.toml`
- `botsettings.template.d.toml`
- `botsettings.template.deploy.toml` (mandatory)
- `botsettings.template.debug.toml` (optional)

To use either `debug` or `deploy` configuration, set the "use" key to true and rebuild:
```toml
# set to true to use this config
use = true

# ... the rest
```

---

## Setup: Telegram
These are the current list of public commands available. Copy and paste these when [setting commands with BotFather:](https://core.telegram.org/bots#botfather-commands)

    help - help
    reload - refresh sheet data
    whoami - who u
    what - what is it?
    whatactually - what is it actually?
    src - view SRC facilities
    namelist - see who's going training
    training - view training program
    paddling - full paddling attendance
    logsheet - SCF logsheet

<!-- countdown - days left to ITCC -->
<!-- weeklybreakdown - attendance breakdown -->

---

<!-- ## Usage: interaction

<img src=".media/canoebot_interaction_512p.gif" alt="Interacting with the bot" width="400"/> -->

## FAQ

### Why did you compile the configuration file into the program? Can't you use an .env file instead for faster rebuilds?
An `.env` file points a key to a string value. Non-string values loose their type information.

Key-value pairs in the config preserve their type information using some codegen trickery, so errors will reveal themselves at compile time instead of runtime.

Changes to config files will trigger a rebuild of the [codegen crate](./crates/ntu_canoebot_config/).

### Why did you switch from python to rust?
Having switched from deploying directly on a VM to using containers, I wanted to create a smaller container image.

The python image is over 300MB in size, while the rust image turned out to be less then 16MB.

The rust version uses less memory too, ~30MB compared to ~150MB.

### Why are there 2 dockerfiles?
Rust has long compile times. Without a cache layer, rebuilding an image downloads and recompiles the same crates more than once.

A solution to this problem is to create a cache layer populated with only cargo manifest files and dummy entry points (`main.rs`, `lib.rs`, `build.rs`). This layer will only need to be rebuilt if any cargo manifest file changes.

TL;DR: build cache layer reduces recompilation time
