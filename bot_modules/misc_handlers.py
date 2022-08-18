'''Definitions for handlers not related to the main function of the bot.'''

import telebot
import time, random
from datetime import date

from bot_modules.common_core import CanoeBot as bot
from bot_modules.common_core import MISC_HANDLERS as MISC_HANDLERS
from bot_modules.common_core import KNOWN_CHATS as KNOWN_CHATS
import modules.utilities as ut

import lib.liblog as lg

## add more commands and update the command list
command_list = {
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

@bot.message_handler(commands=['start'])
def handle_start(message:telebot.types.Message):
    bot.send_message(
        message.chat.id,
        f'Hi {message.from_user.first_name}, this is NTU canoebot! '\
        'Browse the command list or type /help for more detailed instructions.'
    )
    return

## general help
@bot.message_handler(commands=['help'])
@lg.decorators.info()
def handle_help(message:telebot.types.Message):
    helptext:str = 'More about commands:\n\n'

    for key in command_list:
        helptext += "/" + key + ": "
        helptext += command_list[key] + "\n\n"
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
@lg.decorators.info()
def handle_whoami(message:telebot.types.Message):
    lg.functions.debug(f'chat name type: {type(message.from_user)}')
    lg.functions.debug(message.from_user)
    bot.send_message(message.chat.id, str(message.from_user.first_name))

## countdown to ITCC
@bot.message_handler(commands=['countdown'])
def handle_countdown(message:telebot.types.Message):
    text = ' '.join(message.text.split()[1:]) ## new way of stripping command
    bot.send_message(message.chat.id, f'{ut.countdown()} days to ITCC')

## ooga - booga
@bot.message_handler(regexp='ooga')
@lg.decorators.debug()
def misc_oogabooga(message:telebot.types.Message):
    if MISC_HANDLERS['MISC_OOGABOOGA'] is False: return
    bot.send_message(message.chat.id, 'booga')

## marco - polo
@bot.message_handler(regexp='marco')
@lg.decorators.debug()
def misc_marcopolo(message:telebot.types.Message):
    if MISC_HANDLERS['MISC_MARCOPOLO'] is False: return
    bot.send_message(message.chat.id, 'polo')

## ping - pong (only if 'ping' as a word is inside)
@bot.message_handler(func=lambda message: 'ping' in message.text.lower().split())
@lg.decorators.debug()
def misc_pingpong(message:telebot.types.Message):
    if MISC_HANDLERS['MISC_PINGPONG'] is False: return
    bot.send_message(message.chat.id, 'pong')

## die - same tbh
@bot.message_handler(regexp='die')
@lg.decorators.debug()
def misc_dietbh(message:telebot.types.Message):
    if MISC_HANDLERS['MISC_DIETBH'] is False: return
    bot.reply_to(message, 'same tbh')

## plshelp - hell no
@bot.message_handler(func=lambda message: ('please' in message.text.lower()) and 'help' in message.text.lower())
@lg.decorators.debug()
def misc_hellno(message:telebot.types.Message):
    if MISC_HANDLERS['MISC_HELLNO'] is False: return
    bot.reply_to(message,'hell no')

## help - no
@bot.message_handler(regexp='help')
@lg.decorators.debug()
def misc_helpno(message:telebot.types.Message):
    if MISC_HANDLERS['MISC_HELPNO'] is False: return
    bot.reply_to(message, 'no')

## 69 - nice (see below for continuation)
@bot.message_handler(regexp='69')
@lg.decorators.debug()
def misc_69nice(message:telebot.types.Message):
    if MISC_HANDLERS['MISC_69NICE'] is False: return
    bot.reply_to(message, 'nice')

## nice - nice (see above for previous)
@bot.message_handler(func=lambda message: message.text.lower() == 'nice')
@lg.decorators.debug()
def misc_nicenice(message:telebot.types.Message):
    if MISC_HANDLERS['MISC_NICENICE'] is False: return
    bot.reply_to(message, 'nice')

## OSAS
@bot.message_handler(regexp='ovuvuevuevue')
@lg.decorators.debug()
def misc_osas_1(message:telebot.types.Message):
    if MISC_HANDLERS['MISC_OSAS'] is False: return
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
@lg.decorators.debug()
def misc_who_bot(message:telebot.types.Message):
    if MISC_HANDLERS['MISC_WHOBOT'] is False: return

    replies = [
        'good question', 'idk', 'you ask me i ask who?', 'quiet', 'dunno shaddup'
    ]
    bot.reply_to(message, random.choice(replies)) ## random things

## reply if text message is too long
@bot.message_handler(func=lambda message: len(message.text) >= 250)# and len(message.text) <= 550)
@lg.decorators.debug()
def misc_longmsg(message:telebot.types.Message):
    # log.debug("misc long-msg triggered")
    if MISC_HANDLERS['MISC_LONGMSG'] is False: return

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

## Women
@bot.message_handler(regexp=r'(.*\s|^)women$')
@lg.decorators.debug()
def misc_women(message:telebot.types.Message):
    bot.send_message(message.chat.id, f"women {chr(0x2615)}")
    return

## Men
@bot.message_handler(regexp=r'(.*\s|^)men$')
@lg.decorators.debug()
def misc_men(message:telebot.types.Message):
    bot.send_message(message.chat.id, f"men {chr(0x1F37A)}")
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
