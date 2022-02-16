source functions.sh

thing='~/Documents'
test='5 0 * * 0 sudo kill $(pgrep python3)'"&& cd $thing && python3 canoebot.py"
thang='$(pgrep python3)'
echo "$thing"
echo "$test"
echo $thang
echo "zeroth argument passed is $0"
echo "first argument passed is $1"
echo "total no of arguments is $#"
# comment
