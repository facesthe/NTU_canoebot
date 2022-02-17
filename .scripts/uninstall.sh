#!/bin/bash

currpath=$(realpath .)
cd $(dirname $(realpath "$0")) && cd ..
repopath=$(realpath .)
cd $currpath

source $repopath/.scripts/functions.sh
source $repopath/.scripts/data.sh

green=$(tput setaf 2)
rst=$(tput sgr0)

# Removing crontabs and bash aliases
for cronline in "${CRON[@]}"
do
    echo "$green""removing crontab:$rst ${cronline:0:30} ..."
    rm_crontab "$cronline"
done

for aliasline in "${ALIASES[@]}"
do
    echo "$green""removing alias:$rst ${aliasline:0:30} ..."
    rm_bash_alias "$aliasline"
done

source ~/.bashrc
source ~/.bash_aliases
echo "$green""canoebot uninstalled""$rst"
