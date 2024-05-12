#!/bin/sh

trap 'kill -TERM $PID' TERM INT
/usr/local/bin/bablfsh "$@" &
PID=$!
wait $PID
trap - TERM INT
wait $PID
EXIT_STATUS=$?