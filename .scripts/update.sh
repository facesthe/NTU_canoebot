#!/bin/bash

source functions.sh # import functions
source .repopath.sh # import repopath

git_shallow_pull $repopath
