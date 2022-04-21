#!/bin/bash
# modifies settings.py to use botsettings.json

currpath=$(realpath .)
cd $(dirname $(realpath "$0")) && cd ..
repopath=$(realpath .)
cd $repopath/modules

source $repopath/.scripts/functions.sh
source $repopath/.scripts/data.sh

green=$(tput setaf 2)
bold=$(tput bold)
rst=$(tput sgr0)

for filepath in "${FILEPATHS[@]}"
do
    echo "$green""removing filepath:$rst ${filepath:0:30} ..."
    remove_matching_line "$filepath" settings.py
done

append_to_top "_path = './.configs/botsettings.json' ## deployed version" \
settings.py

cd $currpath
echo "$green""$bold""bot configured for deployment""$rst"
