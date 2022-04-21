#!/bin/bash
## lol this script does not work cause the 'source' keyword only
## works in the current shell session (aka the script). Lesson
## learned.

currpath=$(realpath .)
cd $(dirname $(realpath "$0")) && cd ..
repopath=$(realpath .)
cd $repopath

no_opt_taken=1

while getopts 'ad' OPTION; do
    case "$OPTION" in
        a)
            source $repopath/.venv/bin/activate
            no_opt_taken=0
            ;;
        d)
            source $repopath/.venv/bin/activate
            deactivate
            no_opt_taken=0
            ;;
        ?)
            echo "invalid flag"
            ;;
    esac
done

if [[ $no_opt_taken -eq 1 ]]
then
    echo "usage: $(basename $0) [-a / -d]"
    echo "-a: activate python virtual environment"
    echo "-d: deactivate python virtual environment"
fi

cd $currpath
exit 0
