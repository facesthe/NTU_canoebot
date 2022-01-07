from dateutil.parser import parse
import telebot, time, random
from datetime import date#, timedelta

## bot modules
import gymscraper as  gs ## for the wavegym command
import srcscraper as sc ## for srcscraper command (NEW)
import sheetscraper as ss ## for attendance stuffs
import bashcmds as bc ## for interfacing with terminal in the pi
import formfiller as ff ## for sending the log sheet
import debuglogging as dl ## for info and logging
import contacttrace as ct ## for contact tracing
import settings as s ## bot settings

TOKEN = s.json.canoebot.apikey ## API token for the bot
bot = telebot.TeleBot(TOKEN, parse_mode=None)
submit_date = None ## for formfiller submit history

log = dl.log ## logging obj
trace = ct.tracer()

log.info("starting canoebot")

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

## general help
@bot.message_handler(commands=['help'])
def handle_help(message):
    log.info("/help handler triggered")
    helptext = 'Commands:\n\n'
    cid = message.chat.id

    for key in command_list:
        helptext += "/" + key + ": "
        helptext += command_list[key] + "\n"
    bot.send_message(cid, helptext)

## hidden help
@bot.message_handler(commands=['xcohelp'])
def handle_xcohelp(message):
    log.info("/xcohelp handler triggered")
    helptext = 'Hidden commands:\n\n'
    cid = message.chat.id

    for key in command_list_hidden:
        helptext += "/" + key + ": "
        helptext += command_list_hidden[key] + "\n"
    bot.send_message(cid, helptext)

## wavegym command
@bot.message_handler(commands=['wavegym'])
def handle_wavegym(message):
    log.info("/wavegym handler triggered")
    text = ' '.join(message.text.split()[1:]) ## new way of stripping command
    bot.send_chat_action(message.chat.id, "typing")
    bot.send_message(message.chat.id, ss.codeit(gs.response(text)), parse_mode='Markdown')


## src command part 1
@bot.message_handler(commands=['srcbookings'])
def handle_srcbooking_1(message):
    log.info("/srcbooking-1 handler triggered")
    bot.send_message(message.chat.id, "SRC booking lookup! /cancel to return")
    bot.send_message(message.chat.id, ss.codeit(sc.show_facility_table()), parse_mode='Markdown')
    handle_srcbooking_2(message)

## src command part 2
def handle_srcbooking_2(message):
    log.info("/srcbooking-2 handler triggered")

    msg = bot.send_message(message.chat.id, "enter a facility number:")
    bot.register_next_step_handler(msg, handle_srcbooking_3)

## src command part 3
def handle_srcbooking_3(message):
    log.info("/srcbooking-3 handler triggered")
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
            log.debug(f"Input number is out of range")
            bot.send_message(message.chat.id, "number not valid")
            # msg = bot.send_message(message.chat.id, "please enter a valid facility number:")
            # bot.register_next_step_handler(message, handle_srcbooking_2)
            handle_srcbooking_2(message)

    else:
        log.debug(f"invalid input: {text} is not a number")
        bot.send_message(message.chat.id, "not a number")
        # msg = bot.send_message(message.chat.id, "please enter a facility number:")
        # bot.register_next_step_handler(message, handle_srcbooking_2)
        handle_srcbooking_2(message)

## src command part 4
def handle_srcbooking_4(message, tablecol):
    log.info("/srcbooking-4 handler triggered")
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
def handle_namelist(message):
    log.info("/namelist handler triggered")
    text = ' '.join(message.text.split()[1:]) ## new way of stripping command
    try:
        reply = ss.namelist(text)
        bot.send_message(message.chat.id, ss.df2str(reply),parse_mode='Markdown')
    except: ## to catch out-of-range input dates
        bot.send_message(message.chat.id,'Date out of range!')

## sync with contents of the configs sheet
@bot.message_handler(commands=['reload'])
def handle_reload(message):
    log.info("/reload handler triggered")
    ss.updateconfigs()
    bot.send_message(message.chat.id,'updated')

##################################################################################
## Misc commands - these do not contribute to the functionality of the bot
##################################################################################

## enable/disable some annoying handlers
## misc - enable
@bot.message_handler(commands=['enable'])
def misc_enable(message):
    log.info("/enable handler triggered")
    text = ' '.join(message.text.split()[1:]).upper() ## new way of stripping command
    if text in misc_handlers.keys():
        misc_handlers[text] = True
        bot.send_message(message.chat.id, f"{text} enabled")
    else:
        bot.send_message(message.chat.id, "handle not found")

## misc - disable
@bot.message_handler(commands=['disable'])
def misc_disable(message):
    log.info("/disable handler triggered")
    text = ' '.join(message.text.split()[1:]).upper() ## new way of stripping command
    if text in misc_handlers.keys():
        misc_handlers[text] = False
        bot.send_message(message.chat.id, f"{text} disabled")
    else:
        bot.send_message(message.chat.id, "handle not found")

## view status of handler
## misc - status
@bot.message_handler(commands=['handlerstatus'])
def misc_handlerstatus(message):
    log.info("/handlerstatus handler triggered")
    text = ' '.join(message.text.split()[1:]).upper() ## new way of stripping command
    if text in misc_handlers.keys():
        bot.send_message(message.chat.id, misc_handlers[text.upper()])
    else:
        bot.send_message(message.chat.id, "handle not found")

## ooga - booga
@bot.message_handler(regexp='ooga')
def misc_oogabooga(message):
    log.debug("misc ooga-booga triggered")
    if misc_handlers['MISC_OOGABOOGA'] is False: return
    bot.send_message(message.chat.id, 'booga')

## marco - polo
@bot.message_handler(regexp='marco')
def misc_marcopolo(message):
    log.debug("misc marco-polo triggered")
    if misc_handlers['MISC_MARCOPOLO'] is False: return
    bot.send_message(message.chat.id, 'polo')

## ping - pong (only if 'ping' as a word is inside)
@bot.message_handler(func=lambda message: 'ping' in message.text.lower().split())
def misc_pingpong(message):
    log.debug("misc ping-pong triggered")
    if misc_handlers['MISC_PINGPONG'] is False: return
    bot.send_message(message.chat.id, 'pong')

## die - same tbh
@bot.message_handler(regexp='die')
def misc_dietbh(message):
    log.debug("misc die-sametbh triggered")
    if misc_handlers['MISC_DIETBH'] is False: return
    bot.reply_to(message, 'same tbh')

## plshelp - hell no
@bot.message_handler(func=lambda message: ('please' in message.text.lower()) and 'help' in message.text.lower())
def misc_hellno(message):
    log.debug("misc help-hellno triggered")
    if misc_handlers['MISC_HELLNO'] is False: return
    bot.reply_to(message,'hell no')

## help - no
@bot.message_handler(regexp='help')
def misc_helpno(message):
    log.debug("misc help-no triggered")
    if misc_handlers['MISC_HELPNO'] is False: return
    bot.reply_to(message, 'no')

## 69 - nice (see below for continuation)
@bot.message_handler(regexp='69')
def misc_69nice(message):
    log.debug("misc 69-nice triggered")
    if misc_handlers['MISC_69NICE'] is False: return
    bot.reply_to(message, 'nice')

## nice - nice (see above for previous)
@bot.message_handler(func=lambda message: message.text.lower() == 'nice')
def misc_nicenice(message):
    log.debug("misc nice-nice triggered")
    if misc_handlers['MISC_NICENICE'] is False: return
    bot.reply_to(message, 'nice')

## count the number of times 'bot' has been mentioned

##OSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSAS##
## the OSAS message handler
## Ovuvuevuevue Enyetuenwuevue Ugbemugbem Osas is the full name
##OSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSAS##
@bot.message_handler(regexp='ovuvuevuevue')
def misc_osas_1(message):
    log.debug("misc osas-1 triggered")
    if misc_handlers['MISC_OSAS'] is False: return
    msg = bot.reply_to(message, "...i'm listening")
    bot.register_next_step_handler(msg, misc_osas_2)

def misc_osas_2(message):
    log.debug("misc osas-2 triggered")
    if 'enyetuenwuevue' in message.text.lower():
        msg = bot.send_message(message.chat.id, "go on...")
        bot.register_next_step_handler(msg, misc_osas_3)

def misc_osas_3(message):
    log.debug("misc osas-3 triggered")
    if 'ugbemugbem' in message.text.lower():
        msg = bot.send_message(message.chat.id, "almost there...")
        bot.register_next_step_handler(msg, misc_osas_4)
    else:
        bot.send_message(message.chat.id, "why you no how call my name na?")

def misc_osas_4(message):
    log.debug("misc osas-4 triggered")
    if 'osas' in message.text.lower():
        bot.send_message(message.chat.id, "i clapping for u na bratha")
        time.sleep(3)
        bot.send_message(message.chat.id, "you know how call my naem")

##OSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSASOSAS##

## reply if text contains 'bot' and 'who'
@bot.message_handler(func=lambda message: \
    'bot' in message.text.lower() and 'who' in message.text.lower())

def misc_who_bot(message):
    log.debug("misc who-bot triggered")
    if misc_handlers['MISC_WHOBOT'] is False: return

    replies = [
        'good question', 'idk', 'you ask me i ask who?', 'quiet', 'dunno shaddup'
    ]
    bot.reply_to(message, random.choice(replies)) ## random things

## reply if text message is too long
@bot.message_handler(func=lambda message: len(message.text) >= 250)# and len(message.text) <= 550)
def misc_longmsg(message):
    log.debug("misc long-msg triggered")
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

## check logs
@bot.message_handler(commands=['botlog'])
def misc_botlog(message):
    log.warning("/botlog handler triggered")
    bot.send_message(message.chat.id, ss.codeit(bc.botlog()), parse_mode='Markdown')

## bash - DISABLE THIS AFTER TESTING
@bot.message_handler(commands=['bash'])
def misc_bash(message):
    log.warning("/bash handler triggered")
    if misc_handlers['MISC_BASH'] is False: return

    text = ' '.join(message.text.split()[1:]) ## new way of stripping command
    bot.send_message(message.chat.id, bc.bashout(text))

## send messages to specific groups
@bot.message_handler(commands=['send_msg'])
def misc_send_msg(message):
    '''Send message using <chat_name>, <chat text>\n
    Try not to use too often'''
    log.warning("/send_msg handler triggered")
    text = ' '.join(message.text.split()[1:]).rstrip() ## new way of stripping command
    log.debug(text)
    log.debug(text.split(','))
    log.debug(known_chats[text.split(',')[0]])
    try:
        bot.send_chat_action(known_chats[text.split(',')[0]], 'typing')
        log.debug('chat action sent')
        time.sleep(random.randint(1,5)) ## make the bot look like there is some typing going on
        bot.send_message(known_chats[text.split(',')[0]], text.split(',')[1])
        bot.send_message(message.chat.id, 'msg sent')
    except:
        bot.send_message(message.chat.id, 'send unsuccessful')

## send videos to specific groups
@bot.message_handler(commands=['send_vid'])
def misc_send_video(message):
    log.warning("/send_vid handler triggered")
    text = ' '.join(message.text.split()[1:]).rstrip() ## new way of stripping command
    try:
        path = './data/media/'+text.split(',')[1].strip(' ')+'.mp4'
        log.info(path)
        bot.send_chat_action(known_chats[text.split(',')[0]], 'typing')
        time.sleep(random.randint(1,5))
        bot.send_video(known_chats[text.split(',')[0]], data=open(path,'rb'))
        bot.send_message(message.chat.id, 'video sent')
    except:
        bot.send_message(message.chat.id, 'send unsuccessful')


## reply in markdown (for testing purposes)
@bot.message_handler(commands=['send_markdown'])
def misc_send_markdown(message):
    log.warning("/send_markdown handler triggered")
    text = ' '.join(message.text.split()[1:]).rstrip() ## new way of stripping command
    bot.send_message(message.chat.id, text, parse_mode='Markdown')

## reply in formatted code block (also for testing purposes)
@bot.message_handler(commands=['send_code'])
def misc_send_code(message):
    log.warning("/send_code handler triggered")
    text = ' '.join(message.text.split()[1:]).rstrip() ## new way of stripping command
    bot.send_message(message.chat.id, ss.codeit(text), parse_mode='Markdown')

## check uptime (keep this at the bottom of misc commands)
@bot.message_handler(commands=['uptime'])
def misc_uptime(message):
    log.info("/uptime handler triggered")
    bot.send_message(message.chat.id, ss.codeit(bc.uptime()), parse_mode='Markdown')

##################################################################################
## Start of exo commands - exco specific commands (more to come)
##################################################################################
## fetch attendance, with boats
@bot.message_handler(commands=['boatallo'])
def handle_boatallo(message):
    log.info("/boatallo handler triggered")
    text = ' '.join(message.text.split()[1:]) ## new way of stripping command
    bot.send_chat_action(message.chat.id, 'typing')
    try:
        reply = ss.boatallo(text)
        bot.send_message(message.chat.id,ss.df2str(reply),parse_mode='Markdown')
    except:
        bot.send_message(message.chat.id,'Input out of range!')

## part 1/4 of log sheet sending
@bot.message_handler(commands=['logsheet'])
def handle_logsheet(message):
    log.info("/logsheet-1 handler triggered")
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
        log.debug("logsheet attempting send on another day")
        reply = 'WARNING: SEND DATE NOT TODAY\n\n' + reply
    elif logsheet.date == submit_date: ## if sending for same day > once
        log.debug("logsheet attempting send more than once")
        reply = 'WARNING: LOG SHEET SENT BEFORE\n\n' + reply

    msg = bot.send_message(message.chat.id, ss.codeit(reply), parse_mode='Markdown')
    ## call the next function
    bot.register_next_step_handler(msg, handle_logsheet_send)

## part 2/4 of log sheet sending
def handle_logsheet_send(message):
    log.info("/logsheet-2 handler triggered")
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
def handle_logsheet_modify_count(message):
    log.info("/logsheet-3 handler triggered")
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
def handle_logsheet_modify_name(message):
    log.info("/logsheet-4 handler triggered")
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
def handle_traceall_1(message):
    log.info("/trace-1 handler triggered")
    trace.reset()
    msg = bot.send_message(message.chat.id, 'enter date')
    bot.register_next_step_handler(msg, handle_traceall_2)

## contact tracing part 2
def handle_traceall_2(message):
    log.info("/trace-2 handler triggered")
    if 'exit' in message.text.lower():
        bot.send_message(message.chat.id, 'result:')
        bot.send_message(message.chat.id, ss.df2str(trace.returntable()), parse_mode='Markdown')
    else:
        trace.updateset(message.text)
        msg = bot.send_message(message.chat.id, 'enter another date or "exit" to finish')
        bot.register_next_step_handler(msg, handle_traceall_2)



## keep this at the bottom
bot.infinity_polling()#timeout=10, long_polling_timeout=5)
