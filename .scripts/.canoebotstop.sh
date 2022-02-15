#!/bin/bash
sudo kill $(pgrep -f "python3 canoebot.py")
echo "canoebot stopped"
