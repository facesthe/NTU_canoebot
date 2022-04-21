'''Initialises the bot object with all necessary parameters.
'''
import multiprocessing
import telebot

import modules.json_update
modules.json_update.run()

import modules.settings as s

TOKEN = s.json.canoebot.apikey
SYS_THREADS_AVAIL = multiprocessing.cpu_count()

known_chats = s.json.canoebot.known_chats
misc_handlers = s.json.canoebot.misc_handlers

CanoeBot = telebot.TeleBot(
    TOKEN,
    parse_mode=None,
    threaded=True,
    num_threads=SYS_THREADS_AVAIL,
)
