# NTU canoebot
If you are able to access this repo, you are able to download and host your own bot!

---

## Prerequisites

### For Debian/Ubuntu-based systems:
Install the venv package for python3 by running:

`sudo apt install python3-venv`

Install the dependency for lxml by running:

`sudo apt install libxslt-dev`

### For Raspberry Pi (3B and up, zero 2)
Install the numpy c-extensions by running:

`sudo apt install libatlas-base-dev`

---

## Installing
This bot is designed to work on Linux-based systems E.g. Debian, Arch, WSL.

### For developers

Install dependencies in a python virtual environment by running:

`bash .scripts/install_venv.sh`

### For end-users

1. clone this repository:

    `git clone https://github.com/cruzerngz/NTU_canoebot.git`


2. Navigate to the repository using:

    `cd /path/to/this/repo`

3. Run the install script:

    `bash .scripts/install.sh`

    This script installs the necessary python modules in a virtual environment,
    adds bash aliases to start/stop/view logs, and sets up crontabs.

4. Set your host machine's timezone

    **This is important!**
    If timezones are set up incorrectly, scheduled events will not trigger at the correct time.
    Set the timezone by running:

    `sudo timedatectl set-timezone <your-timezone>`

---

## Uninstalling
Navigate to the source directory and run:

`bash .scripts/uninstall.sh`

This removes all installed bash aliases, crontabs and the python virtual environment.

---

## Configuring settings
Now that the bot has been installed, it needs to be set up to point to all the necessary Google resource IDs in order to function properly.
Navigate to the following directory:

`cd .configs && ls -al`

Note the files that contain "template" in them.
These will be the only files that require modification.
<!-- Let's go over what needs to be modified for each file. -->

### botsettings.template.json
This file contains common settings shared between the debug and deployed versions of the bot.
For the most part, this file does not need to be modified.

### botsettings.template.deploy.json
This file contains private parameters (API keys, google sheet IDs).
The example file for reference is `botsettings.template.d.json`.
Create a copy of this file and rename it to `botsettings.template.deploy.json`.
Add the API key for the deployed version of the bot, google forms resource IDs, and set the logging level to "INFO" to show informational logs and above.

### botsettings.template.debug.json
This file also takes reference to `botsettings.template.d.json`.
Follow the same procedure as above, but name the file `botsettings.template.debug.json` instead.
Change the API key to a different bot, and change the log level to "DEBUG".

### Review
Altogether there should now be 4 'template' JSON files inside `.configs`:
- `botsettings.template.json`
- `botsettings.template.d.json`
- `botsettings.template.deploy.json`
- `botsettings.template.debug.json`

During startup, these files will be read and merged, creatiing 2 more files:
- `botsettings.json`: Actual deployed configuration file read in by canoebot
- `botsettings.debug.json`: Debug configuration file

Do not modifiy these two files, they are regenerated every startup.

---

## Usage: command line
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
| `canoebotvenvexit` | Exit the python virtual environment. |

---

## Setup: Telegram
These are the current list of public commands available. Copy and paste these when [setting commands with BotFather:](https://core.telegram.org/bots#botfather-commands)

    help - help
    reload - refresh sheet data
    whoami - who u
    uptime - power on time
    what - what is it?
    whatactually - what is it actually?
    src - view SRC facilities
    countdown - days left to ITCC
    traininglog - daily training log
    namelist - see who's going training
    training - view training program
    paddling - full paddling attendance
    weeklybreakdown - attendance breakdown
    logsheet - SCF logsheet

---

<!-- ## Usage: interaction

<img src=".media/canoebot_interaction_512p.gif" alt="Interacting with the bot" width="400"/> -->
