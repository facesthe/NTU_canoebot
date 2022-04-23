import sys
import time
import signal

import lib.liblog as lg

import canoebot_modules
from canoebot_modules.common_core import CanoeBot as bot

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

bot.infinity_polling()#timeout=10, long_polling_timeout=5)
