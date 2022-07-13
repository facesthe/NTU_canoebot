"""Definitions of periodic/specific tasks to be executed, and related init handlers"""

import threading
import signal
import sys
import time
import json as jsn
from datetime import date

import schedule
import telebot.types as telebot_types

from bot_modules.common_core import CanoeBot as bot
import modules.sheetscraper as ss
import modules.srcscraper as sc
import modules.formfiller as ff
import modules.settings as s

import lib.liblog as lg

EXCO_CHAT: int = s.json.canoebot.exco_chat

canoebot_start_time = time.time()
SCHEDULER_EVENT: threading.Event

def exit_handler(signum, frame):
    '''Performs uptime calculations on signal'''
    global SCHEDULER_EVENT

    SCHEDULER_EVENT.set() ## stop the scheduler

    end_time = time.time()
    up_time = end_time - canoebot_start_time
    up_hours, remainder = divmod(up_time, 3600)
    up_mins, up_secs = divmod(remainder, 60)

    lg.functions.info(
        f'Signal {signum} received. Bot exiting. Up time: '\
        '{:02}:{:02}:{:02}'.format(int(up_hours), int(up_mins), int(up_secs))
    )
    sys.exit("exiting canoebot...")


def init():
    '''Runs non-blocking code on bot startup'''
    global SCHEDULER_EVENT

    ## Fill all cache entries on startup
    threading.Thread(
        target=sc.fill_all_cache_sets_threaded
    ).start()

    ## start the scheduler with its jobs
    enqueue_schedule()
    SCHEDULER_EVENT=start_scheduler()
    return


def enqueue_schedule():
    '''Queues up all events to be executed'''
    # schedule.every().minute.at(":00").do(event_repeat_test)
    # schedule.every().day.at("07:00:00").do(event_daily_logsheet_am)
    schedule.every().day.at("07:00:00").do(event_daily_logsheet_prompt)
    schedule.every().day.at("19:00:00").do(event_daily_attendance_reminder)
    return


def start_scheduler(interval=1) -> threading.Event:
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
                schedule.run_pending()
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

def event_repeat_test():
    print("testing this is the scheduler running every minute")
    return


@lg.decorators.info()
def event_daily_logsheet_am():
    '''*NOT USED* Auto-sends the logsheet, if paddlers are present'''

    date_today_str = date.today().isoformat()
    names = ss.getonlynames(date_today_str, 0)
    if len(names) == 0:
        bot.send_message(EXCO_CHAT, f"Auto logsheet {date_today_str} slot 0 not sent: no paddlers")

    else:
        logsheet = ff.logSheet()
        logsheet.settimeslot(0)
        logsheet.generateform(date_today_str)
        send_code = logsheet.submitform()

        reply = f'Auto logsheet: {date_today_str} slot 0 submitted: code {send_code}'
        bot.send_message(EXCO_CHAT, reply)

    return


@lg.decorators.info()
def event_daily_logsheet_prompt():
    '''Sends the logsheet message to the exco group as if the command was called'''
    log_date = date.today()

    button_names = ['AM','PM']
    cdata = [{
        'name':'lgsht_data_start',
        'date':str(log_date),
        'time':str(i)
        } for i in range(2)
    ]

    buttons = [
        telebot_types.InlineKeyboardButton(
            button_names[i],
            callback_data=jsn.dumps(cdata[i])
        ) for i in range(2)
    ]

    kb = telebot_types.InlineKeyboardMarkup().add(
        buttons[0], buttons[1]
    )

    bot.send_message(
        EXCO_CHAT,
        f'Logsheet: {log_date}',
        reply_markup=kb,
    )

    return


@lg.decorators.info()
def event_daily_attendance_reminder():
    '''Sends a reminder into the exco chat'''

    bot.send_message(
        EXCO_CHAT,
        "Reminder to do allo"
    )

    return
