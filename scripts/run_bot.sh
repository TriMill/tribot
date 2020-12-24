#!/bin/bash

# Script to run the bot, automatically restarts if the bot 
# exits with code 5 or 6 (from the ";restart" command)

cd "$(dirname "$0")"
cd ..
. secrets/envars.sh
RC=5
while [ $RC -ge 5 ] && [ $RC -le 8 ]; do
	if [ $RC -eq 6 ]; then
		echo "[run.sh] Starting with debug output"
		target/release/discord_bot debug
	else
		echo "[run.sh] Starting bot"
		target/release/discord_bot
	fi
	RC=$?
done
echo "[run.sh] Done"
