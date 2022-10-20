#!/bin/bash

cat $INPUT | head -n $(cat $COUNT) > $OUTPUT