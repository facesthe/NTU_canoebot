'''Initialises the bot object with all necessary parameters.
'''
import telebot

import modules.settings as s

TOKEN = s.json.canoebot.apikey
known_chats = s.json.canoebot.known_chats
misc_handlers = s.json.canoebot.misc_handlers

CanoeBot = telebot.TeleBot(
    TOKEN,
    parse_mode=None,
    threaded=True,
)
