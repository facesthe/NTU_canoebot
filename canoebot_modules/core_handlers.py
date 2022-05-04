import json as jsn
import copy
from datetime import date, timedelta
import telebot
from canoebot_modules import keyboards

from canoebot_modules.common_core import CanoeBot as bot
import modules.sheetscraper as ss
import modules.gymscraper as gs
import modules.srcscraper as sc
import modules.utilities as ut
import modules.TrainingLog as tl
import modules.formfiller as ff
import modules.contacttrace as ct
import modules.bashcmds as bc

import lib.liblog as lg

sc.fill_all_cache_sets_threaded() ## populate facility cache
REMOVE_MARKUP_KB = telebot.types.ReplyKeyboardRemove()

## check uptime (keep this at the bottom of util commands)
@bot.message_handler(commands=['uptime'])
@lg.decorators.info()
def misc_uptime(message:telebot.types.Message):
    bot.send_message(message.chat.id, ss.codeit(bc.uptime()), parse_mode='Markdown')

## sync with contents of the configs sheet
@bot.message_handler(commands=['reload'])
@lg.decorators.info()
def handle_reload(message:telebot.types.Message):
    ss.updateconfigs()
    bot.send_message(message.chat.id,'updated')

## wavegym command
@bot.message_handler(commands=['wavegym'])
@lg.decorators.info()
def handle_wavegym(message:telebot.types.Message):
    text = ' '.join(message.text.split()[1:]) ## new way of stripping command
    bot.send_chat_action(message.chat.id, "typing")
    bot.send_message(message.chat.id, ss.codeit(gs.response(text)), parse_mode='Markdown')

## new src command - navigation using callback buttons
@bot.message_handler(commands=['src'])
@lg.decorators.info()
def handle_srcbooking_new(message:telebot.types.Message):
    '''Displays buttons containing all available facilities'''
    reply = "Choose a SRC facility below:"

    src_facility_list = sc.return_facility_list_shortform()

    kb = telebot.types.InlineKeyboardMarkup().add(
        *[
            telebot.types.InlineKeyboardButton(
                src_facility_list[i],
                callback_data=jsn.dumps({
                    "name":"srcbook_select",
                    "index":i
                })
            )
            for i in range(len(src_facility_list))
        ]
    ).add( ## close button
        telebot.types.InlineKeyboardButton(
            "close",
            callback_data="srcbook_close"
        ),
        row_width=1
    )

    bot.send_message(
        message.chat.id,
        reply,
        reply_markup=kb
    )

    ## do a pre-fetch (if any)
    sc.update_existing_cache_entries_threaded()
    return

@bot.callback_query_handler(func=lambda c: "srcbook_restart" in c.data or "srcbook_cal_back" in c.data)
@lg.decorators.info()
def callback_srcbook_restart(call:telebot.types.CallbackQuery):
    '''Displays buttons containing all available facilities'''
    message=call.message
    reply = "Choose a SRC facility below:"

    src_facility_list = sc.return_facility_list_shortform()

    kb = telebot.types.InlineKeyboardMarkup().add(
        *[
            telebot.types.InlineKeyboardButton(
                src_facility_list[i],
                callback_data=jsn.dumps({
                    "name":"srcbook_select",
                    "index":i
                })
            )
            for i in range(len(src_facility_list))
        ]
    ).add( ## close button
        telebot.types.InlineKeyboardButton(
            "close",
            callback_data="srcbook_close"
        ),
        row_width=1
    )

    bot.edit_message_text(
        reply,
        chat_id=message.chat.id,
        message_id=message.message_id,
        reply_markup=kb
    )

    ## do a pre-fetch (if any)
    sc.update_existing_cache_entries_threaded()
    return

@bot.callback_query_handler(func=lambda c: "srcbook_close" in c.data)
@lg.decorators.info()
def callback_srcbook_close(call:telebot.types.CallbackQuery):
    '''Deletes message containing list of facilities'''
    message=call.message
    bot.edit_message_text(
        "/src",
        chat_id=message.chat.id,
        message_id=message.message_id
    )
    return

@bot.callback_query_handler(func=lambda c: 'srcbook_select' in c.data)
@lg.decorators.info()
def callback_srcbook_facility_select(call:telebot.types.CallbackQuery):
    '''shows calendar for picking date'''
    message=call.message
    cdata_call:dict = jsn.loads(call.data)

    callback_data:dict = copy.deepcopy(cdata_call)
    callback_data.pop("name")

    kb = keyboards.calendar_keyboard_gen("srcbook", date.today(), callback_data)

    reply_0 = sc.FACILITY_TABLE[int(cdata_call["index"])]["name"]
    reply = f"{reply_0}\nChoose a date below:"

    bot.edit_message_text(
        reply,
        chat_id=message.chat.id,
        message_id=message.message_id,
        reply_markup=kb
    )

    return

@bot.callback_query_handler(func=lambda c: 'srcbook_date' in c.data)
@lg.decorators.info()
def callback_srcbook_date_select(call:telebot.types.CallbackQuery):
    '''Displays result for specified facility and date'''
    message=call.message
    cdata_call:dict = jsn.loads(call.data)

    target_date = date.fromisoformat(cdata_call["date"])

    reply = sc.get_booking_result_cache(
        target_date,
        int(cdata_call["index"])
    )

    navi_buttons=["<<",">>"]
    other_buttons=["refresh","back"]

    kb = telebot.types.InlineKeyboardMarkup().add( ## row 1, 2 buttons
        *[
            telebot.types.InlineKeyboardButton(
                navi_buttons[i],
                callback_data=jsn.dumps({
                    "name":"srcbook_date",
                    "date":(target_date+timedelta(days=-1)).isoformat() if i==0 else (target_date+timedelta(days=1)).isoformat(),
                    "index":cdata_call["index"]
                })
            ) for i in range(len(navi_buttons))
        ],
        row_width=2
    ).add( ## row 2, 2 buttons
        *[
            telebot.types.InlineKeyboardButton(
                other_buttons[0],
                callback_data=jsn.dumps({
                    "name":"srcbook_refresh",
                    "date":cdata_call["date"],
                    "index":cdata_call["index"]
                })
            ),
            telebot.types.InlineKeyboardButton(
                other_buttons[1],
                callback_data=jsn.dumps({
                    "name":"srcbook_select",
                    "index":cdata_call["index"]
                })
            )
        ],
        row_width=2
    ).add( ## src link
        telebot.types.InlineKeyboardButton(
            "Link to SRC",
            url = sc.SRC_LINK
        ),
        row_width=1
    )

    bot.edit_message_text(
        ss.codeit(reply),
        chat_id=message.chat.id,
        message_id=message.message_id,
        parse_mode='Markdown',
        reply_markup=kb
    )

    return

@bot.callback_query_handler(func=lambda c: "srcbook_refresh" in c.data)
@lg.decorators.info()
def callback_srcbook_refresh(call:telebot.types.CallbackQuery):
    cdata = jsn.loads(call.data)
    sc.update_single_cache_entry(
        int(cdata["index"]),
        sc.get_cache_line_no(
            date.fromisoformat(cdata["date"]),
            int(cdata["index"])
        ),
        True
    )
    callback_srcbook_date_select(call)
    return

## src command part 1
@bot.message_handler(commands=['srcbookings'])
@lg.decorators.info()
def handle_srcbooking_1(message:telebot.types.Message):
    bot.send_message(message.chat.id, "SRC booking lookup! /cancel to return")
    bot.send_message(message.chat.id, ss.codeit(sc.show_facility_table()), parse_mode='Markdown')
    handle_srcbooking_2(message)

## src command part 2
@lg.decorators.info()
def handle_srcbooking_2(message:telebot.types.Message):

    msg = bot.send_message(message.chat.id, "enter a facility number:")
    bot.register_next_step_handler(msg, handle_srcbooking_3)

## src command part 3
@lg.decorators.info()
def handle_srcbooking_3(message:telebot.types.Message):
    text = message.text
    ## exit command
    if text == "/cancel":
        bot.send_message(message.chat.id, "exiting /srcbookings")
        return

    ## input validation
    if text.isdigit():
        tablecol = int(text)
        if tablecol in range(1, len(sc.FACILITY_TABLE)+1): ## in range, proceed
            msg = bot.send_message(message.chat.id, "enter a date (dd mmm or day):")
            bot.register_next_step_handler(msg, handle_srcbooking_4, tablecol)
        else: ## number not in range
            lg.functions.debug(f"Input number is out of range")
            bot.send_message(message.chat.id, "number not valid")
            # msg = bot.send_message(message.chat.id, "please enter a valid facility number:")
            # bot.register_next_step_handler(message, handle_srcbooking_2)
            handle_srcbooking_2(message)

    else:
        lg.functions.debug(f"invalid input: {text} is not a number")
        bot.send_message(message.chat.id, "not a number")
        # msg = bot.send_message(message.chat.id, "please enter a facility number:")
        # bot.register_next_step_handler(message, handle_srcbooking_2)
        handle_srcbooking_2(message)

## src command part 4
@lg.decorators.info()
def handle_srcbooking_4(message:telebot.types.Message, tablecol):
    text = message.text
    ## exit command
    if text == "/cancel":
        bot.send_message(message.chat.id, "exiting /srcbookings")
        return

    date_obj = sc.parse_date(text)
    bot.send_chat_action(message.chat.id, 'typing')
    bot.send_message(message.chat.id, \
        ss.codeit(sc.get_booking_result(date_obj, tablecol-1)), parse_mode='Markdown')

## Inline callback keyboard generator for /namelist, /training and related callback functions
## Creates a 2 by 2 matrix of buttons that can navigate by day, timeslot, and close
@lg.decorators.debug()
def navigation_button_gen(button_keyword:str, date_in:date, time_slot:int)->telebot.types.InlineKeyboardMarkup:
    button_names = ['<<', '>>', 'time', 'close']
    cdata_navigation:dict = [
        {
            'name':f'{button_keyword}_nav',
            'date':(date_in+timedelta(days=-1)).isoformat(),
            'time':time_slot
        },
        {
            'name':f'{button_keyword}_nav',
            'date':(date_in+timedelta(days=1)).isoformat(),
            'time':time_slot
        },
        {
            'name':f'{button_keyword}_time',
            'date':date_in.isoformat(),
            'time':time_slot
        },
        {
            'name':f'{button_keyword}_close'
        }
    ]
    lg.functions.debug(f'cdata_navi: {jsn.dumps(cdata_navigation, indent=2)}')

    buttons = [
                telebot.types.InlineKeyboardButton(
                    button_names[i],
                    callback_data=jsn.dumps(cdata_navigation[i])
                )
                for i in range(4)
            ]

    kb = telebot.types.InlineKeyboardMarkup().add(
        buttons[0], buttons[1], buttons[2], buttons[3],
        row_width=2
    )

    return kb

## fetch attendance, names only
@bot.message_handler(commands=['namelist'])
@lg.decorators.info()
def handle_namelist(message:telebot.types.Message):
    text = ' '.join(message.text.split()[1:]) ## new way of stripping command

    kb = navigation_button_gen('namelist', ut.parsenamelistdate(text), ut.parsenamelisttimeslot(text))

    try:
        reply = ss.namelist(text)
        bot.send_message(
            chat_id=message.chat.id,
            text=ss.df2str(reply),
            parse_mode='Markdown',
            reply_markup=kb
        )
    except: ## to catch out-of-range input dates
        bot.send_message(message.chat.id,'Out of range. Sheet may not yet exist.')

    return

@bot.callback_query_handler(func=lambda c: 'namelist_nav' in c.data)
@lg.decorators.info()
def callback_namelist_nav(call:telebot.types.CallbackQuery):
    message = call.message

    cdata:dict = jsn.loads(call.data)
    namelist_date = date.fromisoformat(cdata['date'])
    namelist_time = int(cdata['time'])

    kb = navigation_button_gen('namelist', namelist_date, namelist_time)

    reply = ss.namelist(f'{str(namelist_date)},{"pm" if (namelist_time == 1) else ""}')

    bot.edit_message_text(
        chat_id=message.chat.id,
        message_id=message.message_id,
        text=ss.df2str(reply),
        parse_mode='Markdown',
        reply_markup=kb
    )

    return

@bot.callback_query_handler(func=lambda c: 'namelist_time' in c.data)
@lg.decorators.info()
def callback_namelist_time(call:telebot.types.CallbackQuery):
    message = call.message
    cdata:dict = jsn.loads(call.data)
    namelist_date = date.fromisoformat(cdata['date'])
    namelist_time = 1 - int(cdata['time'])

    kb = navigation_button_gen('namelist', namelist_date, namelist_time)

    reply = ss.namelist(f'{str(namelist_date)},{"pm" if (namelist_time == 1) else ""}')

    bot.edit_message_text(
        chat_id=message.chat.id,
        message_id=message.message_id,
        text=ss.df2str(reply),
        parse_mode='Markdown',
        reply_markup=kb
    )

    return

@bot.callback_query_handler(func=lambda c: 'namelist_close' in c.data)
@lg.decorators.info()
def callback_namelist_close(call:telebot.types.CallbackQuery):
    message = call.message

    bot.edit_message_text(
        chat_id=message.chat.id,
        message_id=message.message_id,
        text=ss.codeit(message.text),
        parse_mode='Markdown'
    )

    return

## fetch attendance, with boats
@bot.message_handler(commands=['boatallo'])
@lg.decorators.info()
def handle_boatallo(message:telebot.types.Message):
    text = ' '.join(message.text.split()[1:]) ## new way of stripping command
    bot.send_chat_action(message.chat.id, 'typing')
    try:
        reply = ss.boatallo(text)
        bot.send_message(message.chat.id,ss.df2str(reply),parse_mode='Markdown')
    except:
        bot.send_message(message.chat.id,'Input out of range!')

## boatallo and trainingprog with formatting
@bot.message_handler(commands=['paddling'])
@lg.decorators.info()
def handle_paddling(message:telebot.types.Message):
    text = ' '.join(message.text.split()[1:]) ## new way of stripping command
    reply = ss.paddling(text)
    paddling_date = ut.parsedatetonext(text)

    kb = keyboards.generic_kb_gen(
        'update',
        'paddling',
        {"date":paddling_date.isoformat()}
    )

    bot.send_message(
        message.chat.id,
        ss.codeit(reply),
        parse_mode='Markdown',
        reply_markup=kb
    )

@bot.callback_query_handler(func=lambda c: 'paddling' in c.data)
@lg.decorators.info()
def callback_paddling_refresh(call:telebot.types.CallbackQuery):
    message=call.message
    cdata = jsn.loads(call.data)
    paddling_date = cdata["date"]
    lg.functions.debug(f'date: {paddling_date}')

    reply = ss.paddling(paddling_date)

    kb = keyboards.generic_kb_gen(
        'update',
        'paddling',
        {"date":paddling_date}
    )

    bot.edit_message_text(
        text=ss.codeit(reply),
        chat_id=message.chat.id,
        message_id=message.message_id,
        parse_mode='Markdown',
        reply_markup=kb
    )

    return

## view program w/ callbacks to navigate
## replaces trainingam and trainingpm
@bot.message_handler(commands=['training'])
@lg.decorators.info()
def handle_training_prog(message:telebot.types.Message):
    text = ' '.join(message.text.split()[1:]) ## new way of stripping command
    reply = ss.trainingam(text)

    kb = navigation_button_gen('training_prog', ut.parsedatetocurr(text), 0)

    bot.send_message(
        message.chat.id,
        reply,
        parse_mode='Markdown',
        reply_markup=kb
    )

    return

@bot.callback_query_handler(func=lambda c: 'training_prog_nav' in c.data)
@lg.decorators.info()
def callback_training_prog_nav(call:telebot.types.CallbackQuery):
    message = call.message
    cdata:dict = jsn.loads(call.data)
    new_date = ut.parsedatetocurr(cdata['date'])
    new_time = int(cdata['time'])

    kb = navigation_button_gen('training_prog', new_date, new_time)

    reply:str = ''
    if(new_time == 0):
        reply = ss.trainingam(str(new_date))
    else:
        reply = ss.trainingpm(str(new_date))

    bot.edit_message_text(
        chat_id=message.chat.id,
        message_id=message.message_id,
        text=reply,
        parse_mode='Markdown',
        reply_markup=kb
    )

    return

@bot.callback_query_handler(func=lambda c: 'training_prog_time' in c.data)
@lg.decorators.info()
def callback_training_prog_time(call:telebot.types.CallbackQuery):
    message = call.message
    cdata:dict = jsn.loads(call.data)
    new_date = ut.parsedatetocurr(cdata['date'])
    new_time = 1 - int(cdata['time'])

    kb = navigation_button_gen('training_prog', new_date, new_time)

    reply:str = ''
    if(new_time == 0):
        reply = ss.trainingam(str(new_date))
    else:
        reply = ss.trainingpm(str(new_date))

    bot.edit_message_text(
        chat_id=message.chat.id,
        message_id=message.message_id,
        text=reply,
        parse_mode='Markdown',
        reply_markup=kb
    )

    return

@bot.callback_query_handler(func=lambda c: 'training_prog_close' in c.data)
@lg.decorators.info()
def callback_training_prog_close(call:telebot.types.CallbackQuery):
    message = call.message

    bot.edit_message_text(
        chat_id=message.chat.id,
        message_id=message.message_id,
        text=ss.codeit(message.text),
        parse_mode='Markdown'
    )
    return

## fetch training program for the day
@bot.message_handler(commands=['trainingam'])
@lg.decorators.info()
def handle_trainingam(message:telebot.types.Message):
    text = ' '.join(message.text.split()[1:]) ## new way of stripping command
    reply = ss.trainingam(text)
    bot.send_message(message.chat.id, reply, parse_mode='Markdown')

@bot.message_handler(commands=['trainingpm'])
@lg.decorators.info()
def handle_trainingpm(message:telebot.types.Message):
    text = ' '.join(message.text.split()[1:]) ## new way of stripping command
    reply = ss.trainingpm(text)
    bot.send_message(message.chat.id, reply, parse_mode='Markdown')

## re-writing logsheet
@bot.message_handler(commands=['logsheet'])
@lg.decorators.info()
def handle_logsheet_new_start(message:telebot.types.Message):
    text = ' '.join(message.text.split()[1:]) ## new way of stripping command
    log_date = ut.parsedatetocurr(text)

    button_names = ['AM','PM']
    cdata = [{
        'name':'lgsht_data_start',
        'date':str(log_date),
        'time':str(i)
        } for i in range(2)
    ]

    buttons = [
        telebot.types.InlineKeyboardButton(
            button_names[i],
            callback_data=jsn.dumps(cdata[i])
        ) for i in range(2)
    ]

    kb = telebot.types.InlineKeyboardMarkup().add(
        buttons[0], buttons[1]
    )

    bot.send_message(
        message.chat.id,
        f'Logsheet: {log_date}',
        reply_markup=kb,
    )
    return

@bot.callback_query_handler(func=lambda c: 'lgsht_data_start' in c.data)
@lg.decorators.info()
def callback_logsheet_confirm(call:telebot.types.CallbackQuery):
    message = call.message
    lg.functions.debug(f'callback data: {call.data}')
    cdata:dict = jsn.loads(call.data)
    send_cdata = cdata.copy()
    canc_cdata = cdata.copy()
    send_cdata.update({'name':'lgsht_data_send'})
    canc_cdata.update({'name':'lgsht_data_canc'})
    lg.functions.debug(f'send data: {send_cdata}')
    kb = telebot.types.InlineKeyboardMarkup().add(
        telebot.types.InlineKeyboardButton('Send', callback_data=jsn.dumps(send_cdata)),
        telebot.types.InlineKeyboardButton('Cancel', callback_data=jsn.dumps(canc_cdata))
    )

    logsheet = ff.logSheet()
    logsheet.settimeslot(int(cdata['time']))
    logsheet.generateform(cdata['date'])

    reply = f'Logsheet: {cdata["date"]}\n'\
            f'Time: {logsheet.starttime} to {logsheet.endtime}\n'\
            f'Paddlers: {logsheet.star0 + logsheet.star1}'

    bot.edit_message_text(
        chat_id=message.chat.id,
        message_id=message.message_id,
        text=reply,
        reply_markup=kb,
    )

    return

@bot.callback_query_handler(func=lambda c: 'lgsht_data_send' in c.data)
@lg.decorators.info()
def callback_logsheet_send(call:telebot.types.CallbackQuery):
    message = call.message
    cdata =  jsn.loads(call.data)
    lg.functions.debug(f'callback data contents: {call.data}')

    logsheet = ff.logSheet()
    logsheet.settimeslot(int(cdata['time']))
    logsheet.generateform(cdata['date'])
    lg.functions.debug(f'logsheet: {jsn.dumps(logsheet.gForm.fJSON, indent=2)}')

    result = 'successfully' if logsheet.submitform() == 1 else 'unsuccessfully'
    reply = f'Logsheet: {cdata["date"]} slot {cdata["time"]} '\
            f'submitted {result}'

    bot.edit_message_text(
        chat_id=message.chat.id,
        message_id=message.message_id,
        text=reply,
    )

    return

@bot.callback_query_handler(func=lambda c: 'lgsht_data_canc' in c.data)
@lg.decorators.info()
def callback_logsheet_cancel(call:telebot.types.CallbackQuery):
    message = call.message
    cdata = jsn.loads(call.data)
    reply = f'Logsheet: {cdata["date"]} slot {cdata["time"]} cancelled'

    bot.edit_message_text(
        chat_id=message.chat.id,
        message_id=message.message_id,
        text=reply,
    )
    return

## part 1/4 of log sheet sending
@bot.message_handler(commands=['logsheetold'])
@lg.decorators.info()
def handle_logsheet_old(message:telebot.types.Message):
    global form, submit_date, logsheet
    text = ' '.join(message.text.split()[1:]) ## new way of stripping command
    logsheet = ff.logSheet()
    form = logsheet.generateform(text) ## generate the form here

    reply = f'''Send log sheet as: {logsheet.name}
Date: {logsheet.datestr}
Time: {logsheet.starttime} to {logsheet.endtime}
Total paddlers: {logsheet.star0+logsheet.star1}
Do you want to continue? (Y/N)'''

    ## warnings
    if not logsheet.ispresent: ## not sending for present day, continue with warning
        lg.functions.debug("logsheet attempting send on another day")
        reply = 'WARNING: SEND DATE NOT TODAY\n\n' + reply
    elif logsheet.date == submit_date: ## if sending for same day > once
        lg.functions.debug("logsheet attempting send more than once")
        reply = 'WARNING: LOG SHEET SENT BEFORE\n\n' + reply

    msg = bot.send_message(message.chat.id, ss.codeit(reply), parse_mode='Markdown')
    ## call the next function
    bot.register_next_step_handler(msg, handle_logsheet_old_send)

## part 2/4 of log sheet sending
@lg.decorators.info()
def handle_logsheet_old_send(message:telebot.types.Message):
    global form, logsheet, submit_date
    text = message.text

    if text == 'Y':
        val = ff.submitform(form)
        if val:
            bot.send_message(message.chat.id,'log sheet submitted')
            ## perform assignment of submit_date
            if logsheet.date == date.today(): ## only assign if submission date is present
                submit_date = date.today()
        else:
            bot.send_message(message.chat.id,'submission unsuccessful')

    ## cancel send
    elif text == 'N':
        bot.send_message(message.chat.id, 'cancelled')

    ## modify count
    elif text == 'modify-count':
        msg = bot.send_message(message.chat.id, "enter new count:")
        bot.register_next_step_handler(msg, handle_logsheet_old_modify_count)

    ## modify name
    elif text == 'modify-name':
        msg = bot.send_message(message.chat.id, "enter new name:")
        bot.register_next_step_handler(msg, handle_logsheet_modify_name)

    ## go back to start
    else:
        msg = bot.reply_to(message, 'invalid response, try again.')
        bot.register_next_step_handler(msg, handle_logsheet_old_send)

## part 3/4 of log sheet sending (optional)
@lg.decorators.info()
def handle_logsheet_old_modify_count(message:telebot.types.Message):
    global form, logsheet

    try:
        newcount = int(message.text)
    except:
        msg = bot.reply_to(message, 'invalid response, enter a number.')
        bot.register_next_step_handler(msg, handle_logsheet_old_modify_count)

    logsheet.changeattendance(newcount)

    reply = f'''Send log sheet as: {logsheet.name}
Date: {logsheet.datestr}
Time: {logsheet.starttime} to {logsheet.endtime}
Total paddlers: {logsheet.star0+logsheet.star1}
Do you want to continue? (Y/N)'''

    msg = bot.send_message(message.chat.id, ss.codeit(reply), parse_mode='Markdown')
    bot.register_next_step_handler(msg, handle_logsheet_old_send)

## part 4/4 of log sheet sending (optional)
@lg.decorators.info()
def handle_logsheet_modify_name(message:telebot.types.Message):
    global form, logsheet

    logsheet.changename(message.text)

    reply = f'''Send log sheet as: {logsheet.name}
Date: {logsheet.datestr}
Time: {logsheet.starttime} to {logsheet.endtime}
Total paddlers: {logsheet.star0+logsheet.star1}
Do you want to continue? (Y/N)'''

    msg = bot.send_message(message.chat.id, ss.codeit(reply), parse_mode='Markdown')
    bot.register_next_step_handler(msg, handle_logsheet_old_send)

## contact tracing part 1
@bot.message_handler(commands=['trace'])
@lg.decorators.info()
def handle_traceall_1(message:telebot.types.Message):
    trace = ct.tracer()
    trace.reset()
    msg = bot.send_message(message.chat.id, 'enter date')
    bot.register_next_step_handler(msg, handle_traceall_2, trace)

## contact tracing part 2
@lg.decorators.info()
def handle_traceall_2(message:telebot.types.Message, trace:ct.tracer):
    if 'exit' in message.text.lower():
        bot.send_message(message.chat.id, 'result:')
        bot.send_message(message.chat.id, ss.df2str(trace.returntable()), parse_mode='Markdown')
    else:
        trace.updateset(message.text)
        msg = bot.send_message(message.chat.id, 'enter another date or "exit" to finish')
        bot.register_next_step_handler(msg, handle_traceall_2, trace)

################################################################################
## Training Log
################################################################################

## training log part 1 (under construction)
@bot.message_handler(commands=['traininglog'])
@lg.decorators.info()
def handle_traininglog_1(message:telebot.types.Message):
    kb = telebot.types.ReplyKeyboardMarkup(resize_keyboard=True)
    kb.add('/exit')

    bot.send_message(
        message.chat.id,
        "Daily training log",
        reply_markup=kb)

    traininglog = tl.TrainingLog()
    traininglog.fill_name(message.from_user.full_name)
    msg = bot.send_message(message.chat.id, "Date (dd mmm or day):")
    bot.register_next_step_handler(msg, handle_traininglog_2, traininglog)
    return

## training log part 2 (date entry)
@lg.decorators.info()
def handle_traininglog_2(message:telebot.types.Message, traininglog:tl.TrainingLog):
    if message.text == '/exit':
        bot.send_message(message.chat.id, "Exiting /traininglog", reply_markup=REMOVE_MARKUP_KB)
        return

    traininglog.fill_date(traininglog.dateparser(message.text))
    msg = bot.send_message(message.chat.id, "Sleep hours:")
    bot.register_next_step_handler(msg, handle_traininglog_3, traininglog)
    return

## training log part 3 (sleep hours entry)
@lg.decorators.info()
def handle_traininglog_3(message:telebot.types.Message, traininglog:tl.TrainingLog):
    # msg = bot.send_message(message.chat.id, "Sleep hours:")
    if message.text == '/exit':
        bot.send_message(message.chat.id, "Exiting /traininglog", reply_markup=REMOVE_MARKUP_KB)
        return

    try:
        traininglog.fill_sleephr(int(message.text))
    except:
        bot.send_message(message.chat.id, "Invalid, numbers only")
        msg = bot.send_message(message.chat.id, "Sleep hours:")
        bot.register_next_step_handler(msg, handle_traininglog_3, traininglog)
        return

    msg = bot.send_message(message.chat.id, "Energy level (1-10):")
    bot.register_next_step_handler(msg, handle_traininglog_4, traininglog)
    return

## training log part 4 (energy level entry)
@lg.decorators.info()
def handle_traininglog_4(message:telebot.types.Message, traininglog:tl.TrainingLog):
    if message.text == '/exit':
        bot.send_message(message.chat.id, "Exiting /traininglog", reply_markup=REMOVE_MARKUP_KB)
        return

    try:
        energy = int(message.text)
    except:
        bot.send_message(message.chat.id, "Invalid, numbers only.")
        msg = bot.send_message(message.chat.id, "Energy level (1-10):")
        bot.register_next_step_handler(msg, handle_traininglog_4, traininglog)
        return

    if energy not in range(1,11):
        bot.send_message(message.chat.id, "Invalid, 1-10 only.")
        msg = bot.send_message(message.chat.id, "Energy level (1-10):")
        bot.register_next_step_handler(msg, handle_traininglog_4, traininglog)
        return

    traininglog.fill_energy(energy)
    msg = bot.send_message(message.chat.id, "Heart rate (resting):")
    bot.register_next_step_handler(msg, handle_traininglog_5, traininglog)
    return

## training log part 5 (heart rate entry)
@lg.decorators.info()
def handle_traininglog_5(message:telebot.types.Message, traininglog:tl.TrainingLog):
    # msg = bot.send_message(message.chat.id, "Heart rate:")
    if message.text == '/exit':
        bot.send_message(message.chat.id, "Exiting /traininglog", reply_markup=REMOVE_MARKUP_KB)
        return

    try:
        rhr = int(message.text)
    except:
        bot.send_message(message.chat.id, "Invalid, numbers only.")
        msg = bot.send_message(message.chat.id, "Heart rate (resting):")
        bot.register_next_step_handler(msg, handle_traininglog_5, traininglog)
        return

    traininglog.fill_rhr(rhr)
    msg = bot.send_message(message.chat.id, "Paddling mileage (km):")
    bot.register_next_step_handler(msg, handle_traininglog_6, traininglog)
    return

## training log part 6 (mileage entry)
@lg.decorators.info()
def handle_traininglog_6(message:telebot.types.Message, traininglog:tl.TrainingLog):
    if message.text == '/exit':
        bot.send_message(message.chat.id, "Exiting /traininglog", reply_markup=REMOVE_MARKUP_KB)
        return

    try:
        miles = float(message.text)
    except:
        bot.send_message(message.chat.id, "Invalid, numbers only.")
        msg = bot.send_message(message.chat.id, "Paddling mileage (km):")
        bot.register_next_step_handler(msg, handle_traininglog_6, traininglog)
        return

    traininglog.fill_mileage(miles)
    msg = bot.send_message(message.chat.id, "Training comments:", reply_markup=REMOVE_MARKUP_KB)
    bot.register_next_step_handler(msg, handle_traininglog_7, traininglog)
    return

## training log part 7 (comments entry)
@lg.decorators.info()
def handle_traininglog_7(message:telebot.types.Message, traininglog:tl.TrainingLog):

    traininglog.fill_comments(message.text)
    kb = telebot.types.InlineKeyboardMarkup().add(
        telebot.types.InlineKeyboardButton('send', callback_data='traininglog_send_form'),
        telebot.types.InlineKeyboardButton('cancel', callback_data='traininglog_cancel_form')
    )
    msg = bot.send_message(
        message.chat.id,
        str(traininglog),
        reply_markup=kb
    )

    return

@bot.callback_query_handler(func=lambda c: c.data=='traininglog_send_form')
@lg.decorators.info()
def callback_traininglog_send(call:telebot.types.CallbackQuery):
    message = call.message
    traininglog = tl.TrainingLog()
    traininglog.parse_json_data(message.text)

    date_str = str(traininglog.date)
    traininglog.fill_form()
    result = traininglog.submit_form()

    bot.edit_message_text(
        chat_id=message.chat.id,
        message_id=message.message_id,
        text=f'Training log {date_str} submitted: code {result}'
    )

    return

@bot.callback_query_handler(func=lambda c: c.data=='traininglog_cancel_form')
@lg.decorators.info()
def callback_traininglog_cancel(call:telebot.types.CallbackQuery):
    message = call.message
    date_str = jsn.loads(message.text)["date"]

    bot.edit_message_text(
        chat_id=message.chat.id,
        message_id=message.message_id,
        text=f'Training log {date_str} cancelled'
    )

    return
