"""Definitions of periodic/specific tasks to be executed, and related init handlers"""

import threading
import signal
import sys
import time
import json as jsn
from datetime import date, timedelta

import schedule
import telebot.types as telebot_types

import bot_modules
from bot_modules.common_core import CanoeBot as bot
import bot_modules.core_handlers as core
import modules.srcscraper as sc
import modules.settings as s

import lib.liblog as lg

EXCO_CHAT: int = s.json.canoebot.exco_chat

EVENT_SCHEDULER = schedule.Scheduler()
'''Local scheduler instance.
Not to be confused with EVENT_SCHEDULER_FLAG'''

EVENT_SCHEDULER_FLAG: threading.Event

CANOEBOT_START_TIME = time.time()

def exit_handler(signum, frame):
    '''Performs uptime calculations on signal'''
    global EVENT_SCHEDULER_FLAG

    EVENT_SCHEDULER_FLAG.set() ## stop the scheduler

    end_time = time.time()
    up_time = end_time - CANOEBOT_START_TIME
    up_hours, remainder = divmod(up_time, 3600)
    up_mins, up_secs = divmod(remainder, 60)

    lg.functions.info(
        f'Signal {signum} received. Bot exiting. Up time: '\
        '{:02}:{:02}:{:02}'.format(int(up_hours), int(up_mins), int(up_secs))
    )
    sys.exit("exiting canoebot...")


def init():
    '''Runs non-blocking code on bot startup'''
    global EVENT_SCHEDULER_FLAG

    ## Fill all cache entries on startup
    threading.Thread(
        target=sc.fill_all_cache_sets_threaded
    ).start()

    ## start the scheduler with its jobs
    enqueue_schedule()
    EVENT_SCHEDULER_FLAG=start_scheduler(EVENT_SCHEDULER)
    return


def enqueue_schedule():
    '''Queues up all events to be executed.
    Add any periodic events here.
    '''
    # schedule.every().minute.at(":00").do(event_repeat_test)
    # schedule.every().day.at("07:00:00").do(event_daily_logsheet_am)
    EVENT_SCHEDULER.every().day.at("07:00:00").do(event_daily_logsheet_prompt)
    EVENT_SCHEDULER.every().day.at("19:00:00").do(event_daily_attendance_reminder)
    EVENT_SCHEDULER.every().minute.do(event_srcscraper_cache_refresh)
    return


def start_scheduler(scheduler_instance: schedule.Scheduler, interval=1) -> threading.Event:
    '''Continuously run, while executing pending jobs at each
    elapsed time interval.
    @return cease_continuous_run: threading. Event which can
    be set to cease continuous run. Please note that it is
    *intended behavior that run_continuously() does not run
    missed jobs*. For example, if you've registered a job that
    should run every minute and you set a continuous run
    interval of one hour then your job won't be run 60 times
    at each interval but only once.
    '''
    cease_continuous_run = threading.Event()

    class ScheduleThread(threading.Thread):
        @classmethod
        def run(cls):
            while not cease_continuous_run.is_set():
                scheduler_instance.run_pending()
                time.sleep(interval)

    continuous_thread = ScheduleThread()
    continuous_thread.start()
    return cease_continuous_run


def cleanup():
    '''Runs code on exit. Supplements the exit handler.'''
    return


## Relavant signals should call the exit handler
signal.signal(signal.SIGTERM, exit_handler)
signal.signal(signal.SIGINT, exit_handler)


## Start event definitions ##

@lg.decorators.info()
def event_daily_logsheet_prompt():
    '''Sends the logsheet message to the exco group as if the command was called'''
    core.handle_logsheet_new_start(None, EXCO_CHAT)


@lg.decorators.info()
def event_daily_attendance_reminder():
    '''Sends a reminder into the exco chat'''

    kb = telebot_types.InlineKeyboardMarkup().add(
        telebot_types.InlineKeyboardButton(
            "ok lol",
            url=bot_modules.keyboards.RR_LINK
        ),
        telebot_types.InlineKeyboardButton(
            "no",
            callback_data=jsn.dumps({
                "name": "paddling",
                "date": (date.today()+timedelta(days=1)).isoformat()
            })
        )
    )

    bot.send_message(
        chat_id=EXCO_CHAT,
        text="do allo",
        reply_markup=kb
    )

    return

@lg.decorators.info()
def event_srcscraper_cache_refresh():
    sc.update_existing_cache_entries_threaded()
