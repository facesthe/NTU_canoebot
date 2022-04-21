from dateutil.parser import parse
from datetime import date, timedelta

def countdown()->int:
    '''
    Countdown to ITCC.
    ITCC date is 2022-04-09
    '''

    date_now = date.today()
    date_delta = date(2022, 4, 9) - date_now

    return date_delta.days

def printsomething():
    reply = 'asd\n'\
            'asdf'
    print(reply)

def parsedatetocurr(str_in='')->date:
    try:
        return parse(str_in).date()
    except:
        return date.today()

def parsedatetonext(str_in='')->date:
    try:
        return parse(str_in).date()
    except:
        return date.today() + timedelta(days=1)

def parsenamelistdate(str_in='')->date:
    date_time_str = str_in.split(',')
    return parsedatetonext(date_time_str[0])

def parsenamelisttimeslot(str_in='')->int:
    '''Parses the namelist optional second argument.
    Time slot 0 is AM, time slot 1 is PM'''
    date_time_str = str_in.split(',')
    if(len(date_time_str) == 1):
        return 0

    if date_time_str[1].strip().lower() in ['pm','aft','afternoon']:
        return 1
    else:
        return 0
