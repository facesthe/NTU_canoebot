#!/bin/bash
# echo the script name is $(basename "$0")
echo $(dirname $(realpath "$0"))
currpath=$(realpath .)
cd $(dirname $(realpath "$0")) && cd ..
# echo repo directory is $(realpath .)
repopath=$(realpath .)
echo $repopath
cd $currpath
