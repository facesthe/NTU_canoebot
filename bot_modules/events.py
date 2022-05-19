"""Definitions of periodic/specific tasks to be executed, and related init handlers"""

import threading
import signal
import sys
import time

from bot_modules.common_core import CanoeBot as bot
import modules.srcscraper as sc

import lib.liblog as lg

canoebot_start_time = time.time()

def exit_handler(signum, frame):
    '''Performs uptime calculations on signal'''

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


def startup():
    '''Runs non-blocking code on bot startup'''

    ## Fill all cache entries on startup
    threading.Thread(
        target=sc.fill_all_cache_sets_threaded()
    ).start()

