#!/bin/bash
Xephyr -br :1 +xinerama -screen 800x600 -screen 800x600 &
sleep 1
DISPLAY=:1 cargo run &
sleep 1
DISPLAY=:1 xterm &
wait
function on_exit {
	kill $(jobs -p)
}
trap on_exit EXIT
