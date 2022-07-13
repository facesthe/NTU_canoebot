#!/bin/bash
# Bunch of colour shortcuts

black=$(tput setaf 0)
red=$(tput setaf 1)
green=$(tput setaf 2)
blue=$(tput setaf 4)
magenta=$(tput setaf 5)
cyan=$(tput setaf 6)

rst=$(tput sgr0)

# Takes a ANSI colour escape code and colours the text
# Param $1: ANSI colour code
# Param $2: Text to be coloured
echo_col() {
    echo "$1""$2""$rst"
}

echo_black() {
    echo_col "$black" "$1"
}

echo_red() {
    echo_col "$red" "$1"
}

echo_green() {
    echo_col "$green" "$1"
}

echo_blue() {
    echo_col "$blue" "$1"
}

echo_magenta() {
    echo_col "$magenta" "$1"
}

echo_cyan() {
    echo_col "$cyan" "$1"
}
