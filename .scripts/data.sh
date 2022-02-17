#!/bin/bash

# Data file for commonly used stuff
# Currently used by install.sh, uninstall.sh,
# .canoebotdeploy.sh and .canoebotdebug.sh
# Edit cron and bash entries here

# Note that for raw text to be written (no command substitution within $(this bracket) ),
# A string will need to be enclosed in 'single quotes'
# For all shell expansions, use "double quotes"
# Note that bash will read 'adjacent'"strings" as one if on the same line
# No whitespace allowed between adjacent strings
CRON=(
    "@reboot sleep 30 && cd $repopath/.scripts && bash ./.canoebotrestart.sh" # start bot on boot
    "5 0 * * 0 cd $repopath/.scripts && bash ./.canoebotrestart.sh" # wkly restart
) # crontab entries to append

ALIASES=(
    "# start canoebot aliases #"
    "alias canoebotlog='bash $repopath/.scripts/.canoebotlog.sh'" # open stdout log
    "alias canoebotrestart='bash $repopath/.scripts/.canoebotrestart.sh'" # restart bot
    "alias canoebotstop='bash $repopath/.scripts/.canoebotstop.sh'" # stop bot
    "alias canoebotupdate='bash $repopath/.scripts/.canoebotupdate.sh'" # update bot
    "alias canoebotdebug='bash $repopath/.scripts/.canoebotdebug.sh'" # switch to debug mode
    "alias canoebotdeploy='bash $repopath/.scripts/.canoebotdeploy.sh'" # switch to release mode
    "# end canoebot aliases #"
) # bash_aliases to append

FILEPATHS=(
    "# _path = './.configs/botsettings.json' ## deployed version"
    "_path = './.configs/botsettings.json' ## deployed version"
    "# _path = './.configs/botsettings.debug.json' ## debug version"
    "_path = './.configs/botsettings.debug.json' ## debug version"
) # file path assignment used in Settings.py
