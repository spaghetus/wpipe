#!/bin/bash

cat $INPUT_A | read A
cat $INPUT_B | read B

echo $((A + B)) > $SUM
