#!/bin/bash

while cat $INPUT_A | read A && cat $INPUT_B | read B
do
	echo $((A + B)) > $SUM
done
