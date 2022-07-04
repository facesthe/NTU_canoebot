'''Definitions for user-restricted message handlers. Define debug handlers here.'''

from datetime import date
from dateutil.parser import parse
import random, time
import json as jsn

import telebot
from telebot import util
from telebot.callback_data import CallbackData

from bot_modules.common_core import CanoeBot as bot
from bot_modules.common_core import misc_handlers as misc_handlers
from bot_modules.common_core import known_chats as known_chats
import bot_modules.keyboards as keyboards
import modules.sheetscraper as ss
import modules.bashcmds as bc

import lib.liblog as lg

## send user data to logs - restrict use
@bot.message_handler(commands=['ping'])
@lg.decorators.info()
def misc_ping(message:telebot.types.Message):

    kb = telebot.types.InlineKeyboardMarkup().add(
        telebot.types.InlineKeyboardButton(
            "continue",
            callback_data="ping_continue"
        ),
        telebot.types.InlineKeyboardButton(
            "cancel",
            callback_data="ping_cancel"
        )
    )

    bot.send_message(
        message.chat.id,
        "/ping logs the message, its sender, contents and other attributes. "
        "Doing this will expose your username, chat ID and other information. "
        "Do not continue with this action unless explicitly told to.",
        reply_markup=kb
    )
    return

@bot.callback_query_handler(func=lambda c: "ping_continue" in c.data)
def callback_ping_send(call:telebot.types.CallbackQuery):
    message=call.message

    message_unformatted = str(message.chat)
    message_unformatted = message_unformatted.replace("None", "null") ## convert to valid JSON
    message_unformatted = message_unformatted.replace("'", '"') ## replace single with double quotes
    lg.functions.info(jsn.dumps(
        jsn.loads(message_unformatted),
        indent=2
    ))

    bot.edit_message_text(
        "ping sent. this message will be deleted shortly.",
        chat_id=message.chat.id,
        message_id=message.message_id,
    )

    time.sleep(10)

    bot.delete_message(
        chat_id=message.chat.id,
        message_id=message.message_id
    )

    return

@bot.callback_query_handler(func=lambda c: "ping_cancel" in c.data)
@lg.decorators.info()
def callback_ping_cancel(call:telebot.types.CallbackQuery):
    message=call.message

    bot.edit_message_text(
        "ping cancelled. this message will be deleted shortly.",
        chat_id=message.chat.id,
        message_id=message.message_id,
    )

    time.sleep(10)

    bot.delete_message(
        chat_id=message.chat.id,
        message_id=message.message_id
    )

    return

## check logs
@bot.message_handler(commands=['botlog'])
def misc_botlog(message:telebot.types.Message):
    text = ' '.join(message.text.split()[1:]) ## new way of stripping command
    lg.functions.warning(text)

    reply = bc.botlog()
    reply_split = util.smart_split(reply)

    for sub_reply in reply_split:
        bot.send_message(message.chat.id, ss.codeit(sub_reply), parse_mode='Markdown')

## bash - DISABLE THIS AFTER TESTING
@bot.message_handler(commands=['bash'])
def misc_bash(message:telebot.types.Message):
    if misc_handlers['MISC_BASH'] is False:
        lg.functions.warning('command used but no input taken')
        return

    text = ' '.join(message.text.split()[1:]) ## new way of stripping command
    lg.functions.warning(f'bash input: {text}')

    reply = bc.bashout(text)
    reply_split = util.smart_split(reply)

    for sub_reply in reply_split:
        bot.send_message(message.chat.id, ss.codeit(sub_reply), parse_mode='Markdown')

## enable/disable some annoying handlers
## misc - enable
@bot.message_handler(commands=['enable'])
@lg.decorators.info()
def misc_enable(message:telebot.types.Message):
    text = ' '.join(message.text.split()[1:]).upper() ## new way of stripping command
    if text in misc_handlers.keys():
        misc_handlers[text] = True
        bot.send_message(message.chat.id, f"{text} enabled")
    else:
        bot.send_message(message.chat.id, "handle not found")

## misc - disable
@bot.message_handler(commands=['disable'])
@lg.decorators.info()
def misc_disable(message:telebot.types.Message):
    text = ' '.join(message.text.split()[1:]).upper() ## new way of stripping command
    if text in misc_handlers.keys():
        misc_handlers[text] = False
        bot.send_message(message.chat.id, f"{text} disabled")
    else:
        bot.send_message(message.chat.id, "handle not found")

## view status of handler
## misc - status
@bot.message_handler(commands=['handlerstatus'])
@lg.decorators.info()
def misc_handlerstatus(message:telebot.types.Message):
    text = ' '.join(message.text.split()[1:]).upper() ## new way of stripping command
    if text in misc_handlers.keys():
        bot.send_message(message.chat.id, misc_handlers[text.upper()])
    else:
        bot.send_message(message.chat.id, "handle not found")

## send messages to specific groups
@bot.message_handler(commands=['send_msg'])
@lg.decorators.warning()
def misc_send_msg(message:telebot.types.Message):
    '''Send message using <chat_name>, <chat text>\n
    Try not to use too often'''
    text = ' '.join(message.text.split()[1:]).rstrip() ## new way of stripping command
    lg.functions.debug(text)
    lg.functions.debug(text.split(','))
    lg.functions.debug(known_chats[text.split(',')[0]])
    try:
        bot.send_chat_action(known_chats[text.split(',')[0]], 'typing')
        lg.functions.debug('chat action sent')
        time.sleep(random.randint(1,5)) ## make the bot look like there is some typing going on
        bot.send_message(known_chats[text.split(',')[0]], text.split(',')[1])
        bot.send_message(message.chat.id, 'msg sent')
    except:
        bot.send_message(message.chat.id, 'send unsuccessful')

## send videos to specific groups
@bot.message_handler(commands=['send_vid'])
@lg.decorators.warning()
def misc_send_video(message:telebot.types.Message):
    text = ' '.join(message.text.split()[1:]).rstrip() ## new way of stripping command
    try:
        path = './data/media/'+text.split(',')[1].strip(' ')+'.mp4'
        lg.functions.info(path)
        bot.send_chat_action(known_chats[text.split(',')[0]], 'typing')
        time.sleep(random.randint(1,5))
        bot.send_video(known_chats[text.split(',')[0]], data=open(path,'rb'))
        bot.send_message(message.chat.id, 'video sent')
    except:
        bot.send_message(message.chat.id, 'send unsuccessful')

## reply in markdown (for testing purposes)
@bot.message_handler(commands=['send_markdown'])
@lg.decorators.warning()
def misc_send_markdown(message:telebot.types.Message):
    text = ' '.join(message.text.split()[1:]).rstrip() ## new way of stripping command
    bot.send_message(message.chat.id, text, parse_mode='Markdown')

## reply in formatted code block (also for testing purposes)
@bot.message_handler(commands=['send_code'])
@lg.decorators.warning()
def misc_send_code(message:telebot.types.Message):
    text = ' '.join(message.text.split()[1:]).rstrip() ## new way of stripping command
    bot.send_message(message.chat.id, ss.codeit(text), parse_mode='Markdown')

###############################################################
## code testing ##

@bot.message_handler(commands=['markupkeyboard'])
@lg.decorators.debug()
def handle_markupkeyboard(message:telebot.types.Message):
    kb = telebot.types.ReplyKeyboardMarkup(one_time_keyboard=True)
    kb.add('button 1', 'button 2', 'butt haha')
    reply = bot.send_message(message.chat.id, "press somthing please", reply_markup=kb)
    return

## Clearing markup after use
@bot.message_handler(commands=['clearmarkup'])
@lg.decorators.info()
def handle_clearmarkup(message:telebot.types.Message):
    ## remove markup kb
    kb = telebot.types.ReplyKeyboardRemove()
    bot.send_message(message.chat.id, 'clear markup keyboard', reply_markup=kb)
    return

@bot.message_handler(commands=['inlinemarkup'])
@lg.decorators.debug()
def handle_inlinemarkup(message):
    kb = telebot.types.InlineKeyboardMarkup().add(
        telebot.types.InlineKeyboardButton('Option 1', callback_data='inline_yes'),
        telebot.types.InlineKeyboardButton('Option 2', callback_data='inline_no')
    )
    bot.send_message(message.chat.id, 'look at this text bubble!', reply_markup=kb)

## callback handler for callback data 'inline_yes'
@bot.callback_query_handler(func=lambda c: 'inline_' in c.data)
@lg.decorators.debug()
def callback_inline_yes(call: telebot.types.CallbackQuery):
    message = call.message
    bot.edit_message_text(
        chat_id=message.chat.id,
        message_id=message.message_id,
        text='look the messaage changed!')
        ## without a reply markup the edited message will no longer have buttons

button_callback = CallbackData('button_data', prefix='button')

@bot.message_handler(commands=['callbacktest'])
@lg.decorators.debug()
def handle_callback_test(message:telebot.types.Message):
    cdata = button_callback.new(button_data='some string data')
    lg.functions.debug(f'callback data: {cdata}')

    kb = telebot.types.InlineKeyboardMarkup().add(
        telebot.types.InlineKeyboardButton(
            'button',
            callback_data='asd'
        ))

    bot.send_message(message.chat.id, 'new callback test. Press the button.', reply_markup=kb)
    return

@bot.callback_query_handler(func=lambda c: 'asd' == c.data)
@lg.decorators.debug()
def callback_test(call: telebot.types.CallbackQuery):
    message = call.message
    text = message.text

    bot.edit_message_text(
        chat_id=message.chat.id,
        message_id=message.message_id,
        text=f'text modified after callback.\nPrev message: {text}'
    )

    return

@bot.message_handler(commands=['calendar'])
@lg.decorators.debug()
def handle_calendar(message:telebot.types.Message):
    text = ' '.join(message.text.split()[1:]).rstrip() ## new way of stripping command

    try:
        date_in = parse(text).date()
    except:
        date_in = date.today()

    kb = keyboards.calendar_keyboard_gen("calendar", date_in)

    bot.send_message(
        message.chat.id,
        "Select a date:",
        reply_markup=kb
    )

    return

