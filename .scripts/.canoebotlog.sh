#!/bin/bash
# Print log messages to STDOUT
# Bash alias to instal in ~/.bash_aliases:
# alias canoebotlog="sudo bash ~/canoebot/.scripts/.canoebotlog.sh"
sudo tail -f /proc/$(pgrep python3)/fd/1