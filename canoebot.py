'''This is the main file that gets called on canoebot startup. Sparse eh?'''

from bot_modules.common_core import CanoeBot as bot
import bot_modules.events as events

import lib.liblog as lg

lg.functions.info('starting canoebot...')

## run any set events on startup
events.init()

bot.infinity_polling()#timeout=10, long_polling_timeout=5)
