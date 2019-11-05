#!/bin/bash
Xephyr -br :1 &
sleep 1
DISPLAY=:1 cargo run &
sleep 1
DISPLAY=:1 xterm &
wait
function on_exit {
	kill $(jobs -p)
}
trap on_exit EXIT
