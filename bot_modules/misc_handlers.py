'''Definitions for handlers not related to the main function of the bot.'''

import telebot
import time, random
from datetime import date

from bot_modules import helptext as ht
from bot_modules.common_core import CanoeBot as bot
from bot_modules.common_core import MISC_HANDLERS as MISC_HANDLERS
from bot_modules.common_core import KNOWN_CHATS as KNOWN_CHATS
from bot_modules.utilities import verify_exec as verify
import modules.dictionaries as dictionaries
import modules.utilities as ut

import lib.liblog as lg

## add more commands and update the command list
COMMAND_LIST = {
    'reload': 'sync configs sheet',
    'src': 'Interactive SRC viewer',
    'traininglog': 'Telegram frontend for google forms',
    'namelist': 'Interactive name list',
    'training': 'Interactive training programme, subject to availability',
    'logsheet': 'Send daily paddling logsheet to SCF'
    }
## commands for exco only
command_list_hidden = {
    'reload': 'Sync with configs sheet',
    'boatallo': 'Fetch names and allocate boats\nUsage is the same as /namelist',
    'logsheet': 'Sends daily log sheet'
}

## narc prayer para
narc_prayer = [
    "That didn't happen.",
    "And if it did, it wasn't that bad.",
    "And if it was, that's not a big deal.",
    "And if it is, that's not my fault.",
    "And if it was, I didn't mean it.",
    "And if I did, you deserved it."
]

def regex_handler(func_name: str, regex: str, reply: str):
    '''For simple regex-reply handlers. Func name should exist inside
    MISC_HANDLERS if doing conditional execution.'''

    @bot.message_handler(regexp=regex)
    @verify(MISC_HANDLERS, func_name)
    @lg.decorators.debug(func_name)
    def regex_replier(message: telebot.types.Message) -> None:
        bot.reply_to(message, reply)
        return

    regex_replier.__name__ = func_name

    return regex_replier

@bot.message_handler(commands=['start'])
@ht.register_function("/start", True, True)
def handle_start(message:telebot.types.Message):
    '''Receive gratings'''
    bot.send_message(
        message.chat.id,
        f'Hi {message.from_user.first_name}, this is NTU canoebot! '\
        'Browse the command list or type /help for more detailed instructions.'
    )
    return

## general help
@bot.message_handler(commands=['help'])
@ht.register_function("/help", True, True)
@lg.decorators.info()
def handle_help(message:telebot.types.Message):
    '''Show all functions/commands registered with helptext'''
    helptext:str = 'More about commands:\n\n'

    for key in ht.HELP_TEXT_HANDLERS:
        helptext += key + ": "
        helptext += ht.HELP_TEXT_HANDLERS[key] + "\n\n"
    bot.send_message(message.chat.id, helptext)

## hidden help
@bot.message_handler(commands=['xcohelp'])
@lg.decorators.info()
def handle_xcohelp(message:telebot.types.Message):
    helptext = 'Hidden commands:\n\n'
    cid = message.chat.id

    for key in command_list_hidden:
        helptext += "/" + key + ": "
        helptext += command_list_hidden[key] + "\n"
    bot.send_message(cid, helptext)

## echo username - gets the first name of user
@bot.message_handler(commands=['whoami'])
@ht.register_function("/whoami", True, True)
@lg.decorators.info()
def handle_whoami(message:telebot.types.Message):
    '''Returns your telegram username'''
    lg.functions.debug(f'chat name type: {type(message.from_user)}')
    lg.functions.debug(message.from_user)
    bot.send_message(message.chat.id, str(message.from_user.first_name))

## countdown to ITCC
@bot.message_handler(commands=['countdown'])
def handle_countdown(message:telebot.types.Message):
    text = ' '.join(message.text.split()[1:]) ## new way of stripping command
    bot.send_message(message.chat.id, f'{ut.countdown()} days to ITCC')

## Wikipedia series: normal lookup
@bot.message_handler(commands=["what"])
@ht.register_function("/what", True, True)
def misc_wiki_search(message: telebot.types.Message):
    '''Wikipedia API through Telegram'''
    text = ' '.join(message.text.split()[1:]) ## new way of stripping command
    top_resp_summary: str = dictionaries.wiki.summary(text)
    bot.send_message(message.chat.id, top_resp_summary)
    return

## Wikipedia series: drunk lookup
@bot.message_handler(commands=["whatactually"])
@ht.register_function("/whatactually", True, True)
def misc_wiki_search(message: telebot.types.Message):
    '''Urban Dictionary API through Telegram'''
    text = ' '.join(message.text.split()[1:]) ## new way of stripping command
    random_summary: str = dictionaries.urbandictionary.summary(text)
    bot.send_message(message.chat.id, random_summary)
    return


## ooga - booga
ooga_booga = regex_handler("misc_oogabooga", "ooga", "booga")

## marco - polo
marco_polo = regex_handler("misc_marcopolo", "marco", "polo")

## ping - pong (only if 'ping' as a word is inside)
@bot.message_handler(func=lambda message: 'ping' in message.text.lower().split())
@verify(MISC_HANDLERS)
@lg.decorators.debug()
def misc_pingpong(message:telebot.types.Message):
    bot.send_message(message.chat.id, 'pong')

## die - same tbh
die_tbh = regex_handler("misc_dietbh", "die", "same tbh")

## plshelp - hell no
@bot.message_handler(func=lambda message: ('please' in message.text.lower()) and 'help' in message.text.lower())
@lg.decorators.debug()
@verify(MISC_HANDLERS)
def misc_hellno(message:telebot.types.Message):
    bot.reply_to(message,'hell no')

## help - no
help_no = regex_handler("misc_helpno", "help", "no")

## 69 - nice (see below for continuation)
nice = regex_handler("misc_69nice", "69", "nice")

## nice - nice (see above for previous)
nice_nice = regex_handler("misc_nicenice", "nice", "nice")

## OSAS
@bot.message_handler(regexp='ovuvuevuevue')
@verify(MISC_HANDLERS, 'MISC_OSAS')
@lg.decorators.debug()
def misc_osas_1(message:telebot.types.Message):
    msg = bot.reply_to(message, "...i'm listening")
    bot.register_next_step_handler(msg, misc_osas_2)

@lg.decorators.debug()
def misc_osas_2(message:telebot.types.Message):
    if 'enyetuenwuevue' in message.text.lower():
        msg = bot.send_message(message.chat.id, "go on...")
        bot.register_next_step_handler(msg, misc_osas_3)

@lg.decorators.debug()
def misc_osas_3(message:telebot.types.Message):
    if 'ugbemugbem' in message.text.lower():
        msg = bot.send_message(message.chat.id, "almost there...")
        bot.register_next_step_handler(msg, misc_osas_4)
    else:
        bot.send_message(message.chat.id, "why you no how call my name na?")

@lg.decorators.debug()
def misc_osas_4(message:telebot.types.Message):
    if 'osas' in message.text.lower():
        bot.send_message(message.chat.id, "i clapping for u na bratha")
        time.sleep(3)
        bot.send_message(message.chat.id, "you know how call my naem")

## reply if text contains 'bot' and 'who'
@bot.message_handler(func=lambda message: \
    'bot' in message.text.lower() and 'who' in message.text.lower())
@verify(MISC_HANDLERS)
@lg.decorators.debug()
def misc_who_bot(message:telebot.types.Message):

    replies = [
        'good question', 'idk', 'you ask me i ask who?', 'quiet', 'dunno shaddup'
    ]
    bot.reply_to(message, random.choice(replies)) ## random things

## reply if text message is too long
@bot.message_handler(func=lambda message: len(message.text) >= 250)# and len(message.text) <= 550)
@verify(MISC_HANDLERS)
@lg.decorators.debug()
def misc_longmsg(message:telebot.types.Message):
    # log.debug("misc long-msg triggered")

    ## exit if it's the paddling attendance message (list still building)
    keywords = ['paddling','warm']
    for key in keywords:
        if key in message.text.lower():
            return
        else: continue

    delay = int(len(message.text)/20) ## delay is >= 10 seconds
    while delay >= 0: ## keep the typing action on for [delay] seconds
        bot.send_chat_action(message.chat.id, 'typing')
        time.sleep(5)
        delay -= 5

    time.sleep(random.randint(1,int(len(message.text)/20/2))) ## delay again, this time by a rand amt
    bot.reply_to(message, 'K')

## you are already dead - incomplete, memes required
@bot.message_handler(regexp='omae wa mou shindeiru')
def misc_omaewamou(message:telebot.types.Message):
    #if misc_handlers[] is False: return ## key doesnt exist yet
    return
    bot.send_photo(message.chat.id, 'photo_path', 'NANI??') ## upload local file along with a caption

##### bday
@bot.message_handler(regexp='birthday') ## A C T I V A T E only during my bday
def misc_bday(message:telebot.types.Message):
    if date.today() != date(2021,10,16): return
    bday_responses = ['thanks','wow','i get it a lot','arigathanks gozaimuch','ok','m̴̘̲̑̅y̷̭̿ ̴̠̏c̴͚̗̽ỏ̵͍̑n̸̼̕d̶̤̉̾ȏ̷̰͐l̴̥̠̑e̷̮͋͊ͅn̵͍͙͛͂č̴̣̩͝e̶̘͠s̸͕͍͌͂']
    bot.send_message(message.chat.id, random.choice(bday_responses))
##### bday

## deez
feel_deez_nuts = regex_handler("misc_feeldn", r"\bfeel\b", "lol feel deez nuts")

## Women
women = regex_handler("misc_women", r"\bwomen$", f"women {chr(0x2615)}")

## Men
men = regex_handler("misc_men", r"\bmen$", f"men {chr(0x1F37A)}")

## f-pay respects
f_respects = regex_handler("misc_f_respects", r"^f$", f"pay respects {chr(0x1F64F)}")

## used by wwwwwh below
sarcastic_replies_qn: "list(str)" = [
    "dunno",
    "how i know?",
    "how i know sial",
    "i know this, and i'm a bot",
    "sounds like something a kid would know",
    "what a waste of time",
    "how about you go and google that before asking it here",
    "is this some kind of joke?",
    "are you actually looking for an answer or?",
    "you would have figured it out by the time you asked"
]

## 5W 1H series
@bot.message_handler(regexp=r'(?:who|what|when|where|why|how)\b.*\?$')
@verify(MISC_HANDLERS)
@lg.decorators.debug()
def misc_wwwwwh(message:telebot.types.Message):
    if random.randint(0,9): ## 90% change of responding normally
        bot.send_message(message.chat.id, random.choice(sarcastic_replies_qn))

    else: ## 10% chance of doing something else
        if random.randint(0,1):
            bot.send_message(message.chat.id, message.text)
        else:
            return
    return

## ayo
@bot.message_handler(regexp=r'\bayo\b')
@verify(MISC_HANDLERS)
def misc_ayo(message:telebot.types.Message):
    bot.send_message(message.chat.id, f"{chr(0x1F928)}{chr(0x1F4F8)}")
    return

## narcissist's prayer levels - 6 in total
## sends reply to sender of message that was *replied to* with command
@bot.message_handler(commands=['levelone'])
@lg.decorators.info()
def handle_narc_prayer_1(message:telebot.types.Message):
    reply = '\n'.join(narc_prayer[:1])
    try:
        bot.reply_to(message.reply_to_message, reply)
    except:
        lg.functions.debug("No reply message found")
    return

@bot.message_handler(commands=['leveltwo'])
@lg.decorators.info()
def handle_narc_prayer_2(message:telebot.types.Message):
    reply = '\n'.join(narc_prayer[:2])
    try:
        bot.reply_to(message.reply_to_message, reply)
    except:
        lg.functions.debug("No reply message found")
    return

@bot.message_handler(commands=['levelthree'])
@lg.decorators.info()
def handle_narc_prayer_3(message:telebot.types.Message):
    reply = '\n'.join(narc_prayer[:3])
    try:
        bot.reply_to(message.reply_to_message, reply)
    except:
        lg.functions.debug("No reply message found")
    return

@bot.message_handler(commands=['levelfour'])
@lg.decorators.info()
def handle_narc_prayer_4(message:telebot.types.Message):
    reply = '\n'.join(narc_prayer[:4])
    try:
        bot.reply_to(message.reply_to_message, reply)
    except:
        lg.functions.debug("No reply message found")
    return

@bot.message_handler(commands=['levelfive'])
@lg.decorators.info()
def handle_narc_prayer_5(message:telebot.types.Message):
    reply = '\n'.join(narc_prayer[:5])
    try:
        bot.reply_to(message.reply_to_message, reply)
    except:
        lg.functions.debug("No reply message found")
    return

@bot.message_handler(commands=['levelsix'])
@lg.decorators.info()
def handle_narc_prayer_6(message:telebot.types.Message):
    reply = '\n'.join(narc_prayer[:6])
    try:
        bot.reply_to(message.reply_to_message, reply)
    except:
        lg.functions.debug("No reply message found")
    return
