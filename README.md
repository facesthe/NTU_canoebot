# NTU canoebot
If you are able to access this repo, you are able to download and host your own bot!

---

## Prerequisites

```sh
# For Debian/Ubuntu
sudo apt install python3-venv

# lxml dependency
sudo apt install libxslt-dev

# For Raspberry Pis (3B and up, zero 2)
sudo apt install libatlas-base-dev
```

### Docker
[Install docker engine](https://docs.docker.com/engine/install/)

---

## Installing
This bot is designed to work on Linux-based systems E.g. Debian, Arch, WSL.

### For dev

Install if you are running the bot locally without a container:

`bash .scripts/install_venv.sh`

### For end-users

1. clone this repository:

    `git clone https://github.com/cruzerngz/NTU_canoebot.git`


2. Navigate to the repository using:

    `cd /path/to/this/repo`

3. Create config file

    Copy the file `.configs/botsettings.template.d.toml` to
    `.configs/botsettings.template.deploy.toml` and follow the instructions inside.
    [More info here.](#configuring-settings)

4. Build and run container

    `docker compose build`

    `docker compose up -d`

---

## Uninstalling
Navigate to the source directory and run:

`bash .scripts/uninstall.sh`

This removes all installed bash aliases, crontabs and the python virtual environment.

---

## Configuring settings
Before starting the bot, it needs to be set up to point to all the necessary Google resource IDs in order to function properly.

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

To use either `debug` or `deploy` configuration, set the "use" key to true:
```toml
# set to true to use this config
use = true

# ... the rest
```

---

<!-- ## Usage: command line
After running the install script there should be 8 new bash aliases added to your `~/.bash_aliases` file:

You will need to run `source ~/.bash_aliases` once in order for these to show up.

| alias | description |
| --- | --- |
| `canoebotrestart` | Used to start/restart the bot. Any modifications to code/configs should be reflected on restart. |
| `canoebotstop` | Used to stop the bot. |
| `canoebotlog` | Used to show a rolling log output by the bot. Log file located at `.scripts/canoebot.log`. |
| `canoebotupdate` | Performs a git pull with depth 2 from branch "main". |
| `canoebotdeploy` | Switches the bot to deploy settings, API keys, etc. |
| `canoebotdebug` | Switches the bot to debug settings, API keys, etc. **Default setting.** |
| `canoebotvenventer` | Enter the bot's python virtual environment |
| `canoebotvenvexit` | Exit the python virtual environment. | -->

---

## Setup: Telegram
These are the current list of public commands available. Copy and paste these when [setting commands with BotFather:](https://core.telegram.org/bots#botfather-commands)

    help - help
    reload - refresh sheet data
    whoami - who u
    what - what is it?
    whatactually - what is it actually?
    src - view SRC facilities
    countdown - days left to ITCC
    namelist - see who's going training
    training - view training program
    paddling - full paddling attendance
    weeklybreakdown - attendance breakdown
    logsheet - SCF logsheet

---

<!-- ## Usage: interaction

<img src=".media/canoebot_interaction_512p.gif" alt="Interacting with the bot" width="400"/> -->
