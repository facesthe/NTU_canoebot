from datetime import date
from dateutil.relativedelta import relativedelta

def countdown()->int:
    '''
    Countdown to ITCC.
    ITCC date is 2022-04-09
    '''

    date_now = date.today()
    date_delta = date(2022, 4, 9) - date_now

    return date_delta.days


