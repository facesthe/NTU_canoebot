#!/bin/bash
sudo kill 15 $(pgrep python3)
cd /home/pi/canoebot ##dirname needs to change
nohup python3 canoebot.py > /dev/null 2>&1 &
sleep 1
echo
