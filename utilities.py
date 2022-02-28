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
