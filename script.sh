STR1="hello"
STR2="hello"
STR3="asdf"

if [[ $STR1 == $STR2 ]]; then
	echo "STR1 and STR2 match!"
fi

if [[ ($STR1 == $STR2) && -n $STR3 ]]; then
	echo "We should only hit this when there's output below this line"
	echo "$STR3"
fi

