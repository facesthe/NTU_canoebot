#!/bin/bash

currpath=$(realpath .)
cd $(dirname $(realpath "$0")) && cd ..
repopath=$(realpath .)
cd $currpath

source $repopath/.scripts/functions.sh # import functions

git_shallow_pull $repopath
echo "remember to switch to the correct telegram bot!"
echo "defaults to botsettings.debug.json"
