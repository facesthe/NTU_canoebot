'''Main module for canoebot. Called on startup.'''
import sys
import time
import signal

import modules.json_update
modules.json_update.run()

from canoebot_modules.common_core import CanoeBot as bot
import canoebot_modules

import lib.liblog as lg

__author__ = "Ng Jia Rui"
__license__ = "GPL"
__version__ = "0.4.2"
__status__ = "Development"

## keep these at the bottom
canoebot_start_time = time.time()

def exit_handler(signum, frame):
    end_time = time.time()
    up_time = end_time - canoebot_start_time
    up_hours, remainder = divmod(up_time, 3600)
    up_mins, up_secs = divmod(remainder, 60)

    lg.functions.info(
        f'Signal {signum} received. Bot exiting. Up time: '\
        '{:02}:{:02}:{:02}'.format(int(up_hours), int(up_mins), int(up_secs))
    )
    sys.exit("exiting canoebot...")

## Relavant signals should call the exit handler
signal.signal(signal.SIGTERM, exit_handler)
signal.signal(signal.SIGINT, exit_handler)

## polling
bot.infinity_polling()#timeout=10, long_polling_timeout=5)
