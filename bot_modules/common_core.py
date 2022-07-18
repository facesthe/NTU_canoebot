'''Initialises the bot object with all necessary parameters.
'''
import multiprocessing
import telebot

import modules.json_update
modules.json_update.run()

import modules.settings as s

TOKEN = s.json.canoebot.apikey
SYS_THREADS_AVAIL = multiprocessing.cpu_count()

KNOWN_CHATS:dict = s.json.canoebot.known_chats
MISC_HANDLERS:dict = s.json.canoebot.misc_handlers

## remove those pesky connection messages
# telebot.logger.setLevel(telebot.logging.DEBUG)

CanoeBot = telebot.TeleBot(
    TOKEN,
    parse_mode=None,
    threaded=True,
    num_threads=SYS_THREADS_AVAIL,
)
'''Main telebot object. Used to create new message handlers.'''