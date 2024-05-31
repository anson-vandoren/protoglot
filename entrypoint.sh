#!/bin/sh

trap 'kill -TERM $PID' TERM INT
/usr/local/bin/protoglot "$@" &
PID=$!
wait $PID
trap - TERM INT
wait $PID
EXIT_STATUS=$?