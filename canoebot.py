import telebot, time, random
from datetime import date
import json as jsn

## telebot extra classes
from telebot.callback_data import CallbackData

## config constructor, keep at top of imports
import json_update
## bot modules
import gymscraper as  gs ## for the wavegym command
import srcscraper as sc ## for srcscraper command (NEW), may supercede gymscraper
import sheetscraper as ss ## for attendance stuffs
import bashcmds as bc ## for interfacing with terminal in the pi
import formfiller as ff ## for sending the log sheet
from lib.liblog import loggers as lg ## new logging module
import contacttrace as ct ## for contact tracing
import TrainingLog as tl ## for training log
import utilities as ut ## random assortment of functions
import settings as s ## bot settings

TOKEN = s.json.canoebot.apikey ## API token for the bot
bot = telebot.TeleBot(TOKEN, parse_mode=None)
submit_date = None ## for formfiller submit history

trace = ct.tracer()

lg.functions.info(f'using settings file {s._path}')
lg.functions.info("starting canoebot")

## add more commands and update the command list
command_list = {
    'wavegym': 'Fetch current gym availablilty.\nSee other days by using "DD MMM"\n',
    'namelist':'Fetch names for a session. Use <date>, [pm if afternoon]\nDate = "DD MMM"\n',
    'uptime': 'Rpi3 uptime\n'
    }
## commands for exco only
command_list_hidden = {
    'reload': 'Sync with configs sheet\n',
    'namelist': 'Fetch names for a session. Use <date>, [pm if afternoon]\nDate = "DD MMM"\n',
    'boatallo': 'Fetch names and allocate boats\nUsage is the same as /namelist\n',
    'logsheet': 'Send log sheet using a random name\n'
}

## for remotely sending messages through bot
## to retrieve the group/supergroup id, use this url:
## api.telegram.org/bot[api_key]/getUpdates
## enter this in any browser WHILE the bot is NOT active
## find the group/supergroup id inside returned JSON
known_chats = s.json.canoebot.known_chats

## enable/disable misc handlers
misc_handlers = s.json.canoebot.misc_handlers

'y̵̝̝̓e̶̠̬̐̌s̶̻̗̎̊' ## glitch text - for future use

##############################################
################ DECORATORS ##################
##############################################

REMOVE_MARKUP_KB = telebot.types.ReplyKeyboardRemove()

def handleverify(function):
    '''Ensures that bot commands are executed only in known_chats'''
    def wrapper(*args, **kwargs):
        for arg in args:
            if type(arg) == telebot.types.Message:
                if arg.chat.id in s.json.canoebot.known_chats.values():
                    function(*args, **kwargs)
                break
            else:
                lg.functions.warning(
                    'Handler called outside of known chats\n'+
                    f'Called in chat {arg.chat.id}\n'+
                    f'user: {arg.from_user}'
                )
        return

    return wrapper

##############################################
############## DECORATORS-END ################
##############################################

@bot.message_handler(commands=['start'])
def handle_start(message):

    return

## general help
@bot.message_handler(commands=['help'])
@lg.decorators.info()
def handle_help(message):
    helptext = 'Commands:\n\n'
    cid = message.chat.id

    for key in command_list:
        helptext += "/" + key + ": "
        helptext += command_list[key] + "\n"
    bot.send_message(cid, helptext)

## hidden help
@bot.message_handler(commands=['xcohelp'])
@lg.decorators.info()
def handle_xcohelp(message):
    helptext = 'Hidden commands:\n\n'
    cid = message.chat.id

    for key in command_list_hidden:
        helptext += "/" + key + ": "
        helptext += command_list_hidden[key] + "\n"
    bot.send_message(cid, helptext)

## sync with contents of the configs sheet
@bot.message_handler(commands=['reload'])
@lg.decorators.info()
def handle_reload(message):
    ss.updateconfigs()
    bot.send_message(message.chat.id,'updated')

## echo username - gets the first name of user
@bot.message_handler(commands=['whoami'])
@lg.decorators.info()
def handle_whoami(message):
    lg.functions.debug(f'chat name type: {type(message.from_user)}')
    lg.functions.debug(message.from_user)
    bot.send_message(message.chat.id, str(message.from_user.first_name))

## wavegym command
@bot.message_handler(commands=['wavegym'])
@lg.decorators.info()
def handle_wavegym(message):
    text = ' '.join(message.text.split()[1:]) ## new way of stripping command
    bot.send_chat_action(message.chat.id, "typing")
    bot.send_message(message.chat.id, ss.codeit(gs.response(text)), parse_mode='Markdown')

## countdown to ITCC
@bot.message_handler(commands=['countdown'])
def handle_countdown(message):
    text = ' '.join(message.text.split()[1:]) ## new way of stripping command
    bot.send_message(message.chat.id, f'{ut.countdown()} days to ITCC')

## src command part 1
@bot.message_handler(commands=['srcbookings'])
@lg.decorators.info()
def handle_srcbooking_1(message):
    bot.send_message(message.chat.id, "SRC booking lookup! /cancel to return")
    bot.send_message(message.chat.id, ss.codeit(sc.show_facility_table()), parse_mode='Markdown')
    handle_srcbooking_2(message)

## src command part 2
@lg.decorators.info()
def handle_srcbooking_2(message):

    msg = bot.send_message(message.chat.id, "enter a facility number:")
    bot.register_next_step_handler(msg, handle_srcbooking_3)

## src command part 3
@lg.decorators.info()
def handle_srcbooking_3(message):
    text = message.text
    ## exit command
    if text == "/cancel":
        bot.send_message(message.chat.id, "exiting /srcbookings")
        return

    ## input validation
    if text.isdigit():
        tablecol = int(text)
        if tablecol in range(1, len(sc.config)+1): ## in range, proceed
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
def handle_srcbooking_4(message, tablecol):
    text = message.text
    ## exit command
    if text == "/cancel":
        bot.send_message(message.chat.id, "exiting /srcbookings")
        return

    date_obj = sc.parse_date(text)
    bot.send_chat_action(message.chat.id, 'typing')
    bot.send_message(message.chat.id, \
        ss.codeit(sc.get_booking_result(date_obj, tablecol-1)), parse_mode='Markdown')

## fetch attendance, names only
@bot.message_handler(commands=['namelist'])
@lg.decorators.info()
def handle_namelist(message):
    text = ' '.join(message.text.split()[1:]) ## new way of stripping command
    try:
        reply = ss.namelist(text)
        bot.send_message(message.chat.id, ss.df2str(reply),parse_mode='Markdown')
    except: ## to catch out-of-range input dates
        bot.send_message(message.chat.id,'Out of range. Sheet may not yet exist.')

## fetch attendance, with boats
@bot.message_handler(commands=['boatallo'])
@lg.decorators.info()
def handle_boatallo(message):
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
def handle_paddling(message):
    text = ' '.join(message.text.split()[1:]) ## new way of stripping command
    reply = ss.paddling(text)
    bot.send_message(message.chat.id, ss.codeit(reply), parse_mode='Markdown')

## fetch training program for the day
@bot.message_handler(commands=['trainingam'])
@lg.decorators.info()
def handle_trainingam(message):
    text = ' '.join(message.text.split()[1:]) ## new way of stripping command
    reply = ss.trainingam(text)
    bot.send_message(message.chat.id, reply, parse_mode='Markdown')

@bot.message_handler(commands=['trainingpm'])
@lg.decorators.info()
def handle_trainingpm(message):
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
    logsheet = ff.logSheet()
    logsheet.settimeslot(cdata['time'])
    logsheet.generateform(cdata['date'])

    result = 'successfully' if ff.submitform(logsheet.form) == 1 else 'unsuccessfully'
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
def handle_logsheet(message):
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
    bot.register_next_step_handler(msg, handle_logsheet_send)

## part 2/4 of log sheet sending
@lg.decorators.info()
def handle_logsheet_send(message):
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
        bot.register_next_step_handler(msg, handle_logsheet_modify_count)

    ## modify name
    elif text == 'modify-name':
        msg = bot.send_message(message.chat.id, "enter new name:")
        bot.register_next_step_handler(msg, handle_logsheet_modify_name)

    ## go back to start
    else:
        msg = bot.reply_to(message, 'invalid response, try again.')
        bot.register_next_step_handler(msg, handle_logsheet_send)

## part 3/4 of log sheet sending (optional)
@lg.decorators.info()
def handle_logsheet_modify_count(message):
    global form, logsheet

    try:
        newcount = int(message.text)
    except:
        msg = bot.reply_to(message, 'invalid response, enter a number.')
        bot.register_next_step_handler(msg, handle_logsheet_modify_count)

    logsheet.changeattendance(newcount)

    reply = f'''Send log sheet as: {logsheet.name}
Date: {logsheet.datestr}
Time: {logsheet.starttime} to {logsheet.endtime}
Total paddlers: {logsheet.star0+logsheet.star1}
Do you want to continue? (Y/N)'''

    msg = bot.send_message(message.chat.id, ss.codeit(reply), parse_mode='Markdown')
    bot.register_next_step_handler(msg, handle_logsheet_send)

## part 4/4 of log sheet sending (optional)
@lg.decorators.info()
def handle_logsheet_modify_name(message):
    global form, logsheet

    logsheet.changename(message.text)

    reply = f'''Send log sheet as: {logsheet.name}
Date: {logsheet.datestr}
Time: {logsheet.starttime} to {logsheet.endtime}
Total paddlers: {logsheet.star0+logsheet.star1}
Do you want to continue? (Y/N)'''

    msg = bot.send_message(message.chat.id, ss.codeit(reply), parse_mode='Markdown')
    bot.register_next_step_handler(msg, handle_logsheet_send)

## contact tracing part 1
@bot.message_handler(commands=['trace'])
@lg.decorators.info()
def handle_traceall_1(message):
    trace.reset()
    msg = bot.send_message(message.chat.id, 'enter date')
    bot.register_next_step_handler(msg, handle_traceall_2)

## contact tracing part 2
@lg.decorators.info()
def handle_traceall_2(message):
    if 'exit' in message.text.lower():
        bot.send_message(message.chat.id, 'result:')
        bot.send_message(message.chat.id, ss.df2str(trace.returntable()), parse_mode='Markdown')
    else:
        trace.updateset(message.text)
        msg = bot.send_message(message.chat.id, 'enter another date or "exit" to finish')
        bot.register_next_step_handler(msg, handle_traceall_2)

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

##################################################################################
## util commands
##################################################################################

## check logs
@bot.message_handler(commands=['botlog'])
def misc_botlog(message):
    text = ' '.join(message.text.split()[1:]) ## new way of stripping command
    lg.functions.warning(text)
    bot.send_message(message.chat.id, ss.codeit(bc.botlog()), parse_mode='Markdown')

## bash - DISABLE THIS AFTER TESTING
@bot.message_handler(commands=['bash'])
def misc_bash(message):
    if misc_handlers['MISC_BASH'] is False:
        lg.functions.warning('command used but no input taken')
        return

    text = ' '.join(message.text.split()[1:]) ## new way of stripping command
    lg.functions.warning(f'bash input: {text}')
    bot.send_message(message.chat.id, bc.bashout(text))

## enable/disable some annoying handlers
## misc - enable
@bot.message_handler(commands=['enable'])
@lg.decorators.info()
def misc_enable(message):
    text = ' '.join(message.text.split()[1:]).upper() ## new way of stripping command
    if text in misc_handlers.keys():
        misc_handlers[text] = True
        bot.send_message(message.chat.id, f"{text} enabled")
    else:
        bot.send_message(message.chat.id, "handle not found")

## misc - disable
@bot.message_handler(commands=['disable'])
@lg.decorators.info()
def misc_disable(message):
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
def misc_handlerstatus(message):
    text = ' '.join(message.text.split()[1:]).upper() ## new way of stripping command
    if text in misc_handlers.keys():
        bot.send_message(message.chat.id, misc_handlers[text.upper()])
    else:
        bot.send_message(message.chat.id, "handle not found")

## send messages to specific groups
@bot.message_handler(commands=['send_msg'])
@lg.decorators.warning()
def misc_send_msg(message):
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
def misc_send_video(message):
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
def misc_send_markdown(message):
    text = ' '.join(message.text.split()[1:]).rstrip() ## new way of stripping command
    bot.send_message(message.chat.id, text, parse_mode='Markdown')

## reply in formatted code block (also for testing purposes)
@bot.message_handler(commands=['send_code'])
@lg.decorators.warning()
def misc_send_code(message):
    text = ' '.join(message.text.split()[1:]).rstrip() ## new way of stripping command
    bot.send_message(message.chat.id, ss.codeit(text), parse_mode='Markdown')

## check uptime (keep this at the bottom of util commands)
@bot.message_handler(commands=['uptime'])
@lg.decorators.info()
def misc_uptime(message):
    bot.send_message(message.chat.id, ss.codeit(bc.uptime()), parse_mode='Markdown')

##################################################################################
## Misc commands - these do not contribute to the functionality of the bot
##################################################################################

## ooga - booga
@bot.message_handler(regexp='ooga')
@lg.decorators.debug()
def misc_oogabooga(message):
    if misc_handlers['MISC_OOGABOOGA'] is False: return
    bot.send_message(message.chat.id, 'booga')

## marco - polo
@bot.message_handler(regexp='marco')
@lg.decorators.debug()
def misc_marcopolo(message):
    if misc_handlers['MISC_MARCOPOLO'] is False: return
    bot.send_message(message.chat.id, 'polo')

## ping - pong (only if 'ping' as a word is inside)
@bot.message_handler(func=lambda message: 'ping' in message.text.lower().split())
@lg.decorators.debug()
def misc_pingpong(message):
    if misc_handlers['MISC_PINGPONG'] is False: return
    bot.send_message(message.chat.id, 'pong')

## die - same tbh
@bot.message_handler(regexp='die')
@lg.decorators.debug()
def misc_dietbh(message):
    if misc_handlers['MISC_DIETBH'] is False: return
    bot.reply_to(message, 'same tbh')

## plshelp - hell no
@bot.message_handler(func=lambda message: ('please' in message.text.lower()) and 'help' in message.text.lower())
@lg.decorators.debug()
def misc_hellno(message):
    if misc_handlers['MISC_HELLNO'] is False: return
    bot.reply_to(message,'hell no')

## help - no
@bot.message_handler(regexp='help')
@lg.decorators.debug()
def misc_helpno(message):
    if misc_handlers['MISC_HELPNO'] is False: return
    bot.reply_to(message, 'no')

## 69 - nice (see below for continuation)
@bot.message_handler(regexp='69')
@lg.decorators.debug()
def misc_69nice(message):
    if misc_handlers['MISC_69NICE'] is False: return
    bot.reply_to(message, 'nice')

## nice - nice (see above for previous)
@bot.message_handler(func=lambda message: message.text.lower() == 'nice')
@lg.decorators.debug()
def misc_nicenice(message):
    if misc_handlers['MISC_NICENICE'] is False: return
    bot.reply_to(message, 'nice')

## count the number of times 'bot' has been mentioned

##OSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSAS##
## the OSAS message handler
## Ovuvuevuevue Enyetuenwuevue Ugbemugbem Osas is the full name
##OSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSAS##
@bot.message_handler(regexp='ovuvuevuevue')
@lg.decorators.debug()
def misc_osas_1(message):
    if misc_handlers['MISC_OSAS'] is False: return
    msg = bot.reply_to(message, "...i'm listening")
    bot.register_next_step_handler(msg, misc_osas_2)

@lg.decorators.debug()
def misc_osas_2(message):
    if 'enyetuenwuevue' in message.text.lower():
        msg = bot.send_message(message.chat.id, "go on...")
        bot.register_next_step_handler(msg, misc_osas_3)

@lg.decorators.debug()
def misc_osas_3(message):
    if 'ugbemugbem' in message.text.lower():
        msg = bot.send_message(message.chat.id, "almost there...")
        bot.register_next_step_handler(msg, misc_osas_4)
    else:
        bot.send_message(message.chat.id, "why you no how call my name na?")

@lg.decorators.debug()
def misc_osas_4(message):
    if 'osas' in message.text.lower():
        bot.send_message(message.chat.id, "i clapping for u na bratha")
        time.sleep(3)
        bot.send_message(message.chat.id, "you know how call my naem")

##OSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSAS##

## reply if text contains 'bot' and 'who'
@bot.message_handler(func=lambda message: \
    'bot' in message.text.lower() and 'who' in message.text.lower())
@lg.decorators.debug()
def misc_who_bot(message):
    if misc_handlers['MISC_WHOBOT'] is False: return

    replies = [
        'good question', 'idk', 'you ask me i ask who?', 'quiet', 'dunno shaddup'
    ]
    bot.reply_to(message, random.choice(replies)) ## random things

## reply if text message is too long
@bot.message_handler(func=lambda message: len(message.text) >= 250)# and len(message.text) <= 550)
@lg.decorators.debug()
def misc_longmsg(message):
    # log.debug("misc long-msg triggered")
    if misc_handlers['MISC_LONGMSG'] is False: return

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
def misc_omaewamou(message):
    #if misc_handlers[] is False: return ## key doesnt exist yet
    return
    bot.send_photo(message.chat.id, 'photo_path', 'NANI??') ## upload local file along with a caption

##### bday
@bot.message_handler(regexp='birthday') ## A C T I V A T E only during my bday
def misc_bday(message):
    if date.today() != date(2021,10,16): return
    bday_responses = ['thanks','wow','i get it a lot','arigathanks gozaimuch','ok','m̴̘̲̑̅y̷̭̿ ̴̠̏c̴͚̗̽ỏ̵͍̑n̸̼̕d̶̤̉̾ȏ̷̰͐l̴̥̠̑e̷̮͋͊ͅn̵͍͙͛͂č̴̣̩͝e̶̘͠s̸͕͍͌͂']
    bot.send_message(message.chat.id, random.choice(bday_responses))
##### bday

narc_prayer = [
    "That didn't happen.",
    "And if it did, it wasn't that bad.",
    "And if it was, that's not a big deal.",
    "And if it is, that's not my fault.",
    "And if it was, I didn't mean it.",
    "And if I did, you deserved it."
]

## narcissist's prayer levels - 6 in total
@bot.message_handler(commands=['levelone'])
@lg.decorators.info()
def handle_narc_prayer_1(message:telebot.types.Message):
    reply = '\n'.join(narc_prayer[:1])
    bot.reply_to(message, reply)
    return

@bot.message_handler(commands=['leveltwo'])
@lg.decorators.info()
def handle_narc_prayer_2(message:telebot.types.Message):
    reply = '\n'.join(narc_prayer[:2])
    bot.reply_to(message, reply)
    return

@bot.message_handler(commands=['levelthree'])
@lg.decorators.info()
def handle_narc_prayer_3(message:telebot.types.Message):
    reply = '\n'.join(narc_prayer[:3])
    bot.reply_to(message, reply)
    return

@bot.message_handler(commands=['levelfour'])
@lg.decorators.info()
def handle_narc_prayer_4(message:telebot.types.Message):
    reply = '\n'.join(narc_prayer[:4])
    bot.reply_to(message, reply)
    return

@bot.message_handler(commands=['levelfive'])
@lg.decorators.info()
def handle_narc_prayer_5(message:telebot.types.Message):
    reply = '\n'.join(narc_prayer[:5])
    bot.reply_to(message, reply)
    return

@bot.message_handler(commands=['levelsix'])
@lg.decorators.info()
def handle_narc_prayer_6(message:telebot.types.Message):
    reply = '\n'.join(narc_prayer[:6])
    bot.reply_to(message, reply)
    return

###############################################################
## code testing ##

@bot.message_handler(commands=['markupkeyboard'])
@lg.decorators.debug()
def handle_markupkeyboard(message):
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

## keep this at the bottom
bot.infinity_polling()#timeout=10, long_polling_timeout=5)
