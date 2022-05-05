'''Module containing various keyboards and generic callbacks (if any)'''

from  datetime import date, timedelta
from dateutil.relativedelta import relativedelta
from dateutil.parser import parse
import json  as jsn
import copy

import telebot

from canoebot_modules.common_core import CanoeBot as bot
NULL_STR = "None"
RR_LINK = "https://www.youtube.com/watch?v=dQw4w9WgXcQ"
RR_LINK = "None"

import lib.liblog as lg

@bot.callback_query_handler(func=lambda c: "None" in c.data)
@lg.decorators.debug("Callback diverted to null handler")
def callback_none(call:telebot.types.CallbackQuery):

    return

@bot.callback_query_handler(func=lambda c: "_cal_navi" in c.data)
@lg.decorators.debug()
def callback_calendar_navi(call: telebot.types.CallbackQuery):
    '''Generic calendar navigation. Forwards attached callback data.'''
    message = call.message
    cdata:dict = jsn.loads(call.data)

    ## extract original callback data
    callback_data = copy.deepcopy(cdata)
    callback_data.pop("name")
    callback_data.pop("date")

    lg.functions.debug(f"raw cdata: {jsn.dumps(cdata, indent=4)}")
    lg.functions.debug(f"callback data to append: {jsn.dumps(callback_data, indent=4)}")
    new_date = date.fromisoformat(cdata["date"])

    button_keyword:str = cdata["name"]
    button_keyword = button_keyword.replace("_cal_navi", "", 1)

    kb = calendar_keyboard_gen(button_keyword, new_date, callback_data)

    bot.edit_message_text(
        text = message.text,
        chat_id=message.chat.id,
        message_id=message.message_id,
        parse_mode="Markdown",
        reply_markup=kb
    )
    return

def generic_kb_gen(button_name:str, button_keyword:str, callback_data:dict)->telebot.types.InlineKeyboardMarkup:
    '''Generates a keyboard with one key'''
    kb = telebot.types.InlineKeyboardMarkup()

    cdata = {
        "name": button_keyword
    }
    cdata.update(callback_data)

    kb.add(
        telebot.types.InlineKeyboardButton(
            button_name,
            callback_data=jsn.dumps(cdata)
        )
    )

    return kb

def calendar_keyboard_gen(
        button_keyword:str,
        date_in:date,
        callback_data:dict=None,
        include_back_button:bool=True
    )->telebot.types.InlineKeyboardMarkup:
    '''Generates a calendar of a certain month. Attaches general callback data (optional) to every day-number'''

    title:str = date_in.strftime("%b %Y")
    lg.functions.debug(f'date: {date_in}')
    lg.functions.debug(f'callback data passed: {callback_data}')

    header = ["<<",title,">>"]

    cdata_header:list = [
        {
            "name":f"{button_keyword}_cal_navi",
            "date":(date(date_in.year, date_in.month, 1)-relativedelta(months=1)).isoformat()
        },
        {
            "name":NULL_STR
        },
        {
            "name":f"{button_keyword}_cal_navi",
            "date":(date(date_in.year, date_in.month, 1)+relativedelta(months=1)).isoformat()
        }
    ]
    if callback_data is not None:
        cdata_header[0].update(callback_data)
        cdata_header[2].update(callback_data)

    header_buttons = [
        telebot.types.InlineKeyboardButton(
            header[i],
            callback_data=jsn.dumps((cdata_header[i]))
        )
        for i in range(3)
    ]

    ## add header
    kb = telebot.types.InlineKeyboardMarkup().add(
        *header_buttons, row_width=3
    )

    days = ["Mo","Tu","We","Th","Fr","Sa","Su"]
    days_buttons = [telebot.types.InlineKeyboardButton(day, callback_data=RR_LINK) for day in days]

    ## add days using python's 'splat' operator
    kb.add(
        *days_buttons, row_width=7
    )

    month_pre_padding = date(date_in.year, date_in.month, 1).weekday()
    month_num_days = (
        date(date_in.year, date_in.month, 1)
        + relativedelta(months=1)
        + timedelta(days=-1)
        ).day
    month_post_padding:int = (7 - (month_pre_padding + month_num_days)%7 ) % 7

    lg.functions.debug(f"pre padding: {month_pre_padding}", )
    lg.functions.debug(f"num days: {month_num_days}" )
    lg.functions.debug(f"post padding: {month_post_padding}")

    month_buttons = []
    for i in range(month_pre_padding):
        month_buttons.append(
            telebot.types.InlineKeyboardButton(
                " ", callback_data=RR_LINK
            )
        )

    ## adding numbered days
    for i in range(1, month_num_days+1):
        cdata_temp={
            "name":f"{button_keyword}_date",
            "date":date(date_in.year, date_in.month, i).isoformat()
        }
        if callback_data is not None:
            cdata_temp.update(callback_data)

        if date.today().month == date_in.month and date.today().day == i:
            month_buttons.append(
                telebot.types.InlineKeyboardButton(
                    f"{i}\u0332",
                    callback_data=jsn.dumps(cdata_temp)
                )
            )
        else:
            month_buttons.append(
                telebot.types.InlineKeyboardButton(
                    i,
                    callback_data=jsn.dumps(cdata_temp)
                )
            )

    for i in range(month_post_padding):
        month_buttons.append(
            telebot.types.InlineKeyboardButton(
                " ", callback_data=RR_LINK
            )
    )

    kb.add(
        *month_buttons, row_width=7
    )

    if include_back_button:
        back_button_callback_data = {
        "name":f"{button_keyword}_cal_back",
        }
        back_button_callback_data.update(callback_data)

        kb.add(
            telebot.types.InlineKeyboardButton( ## back button
                "back",
                callback_data=jsn.dumps(back_button_callback_data)
            ),
            row_width=1
        )

    return kb


