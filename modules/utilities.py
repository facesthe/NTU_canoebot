'''Miscellaneous helper functions, used primarily in message handlers'''
import os
import re
import random

from dateutil.parser import parse
from datetime import date, timedelta

import lib.liblog as lg

FILLER_CHAR_ARR = [chr(uni_code) for uni_code in range(0x2580, 0x25A0)]

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

def replace_with_placeholder(target: str, filler: str)->str:
    return re.sub(r"[^\s]", filler, target)

def replace_with_placeholder_random(target: str)->str:
    return replace_with_placeholder(target, random.choice(FILLER_CHAR_ARR))

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

def mkdirs_from_dict(dict_in: dict):
    '''Recursive mkdir function. Takes in a multilevel dict and recursively creates directories.
    Note that all keys must be strings (directory names) and all values must be dictionaries.
    The lowest directory key must have an empty dictionary as a value.'''

    ## base case: empty dictionary as value in key-val pair
    for key, val in dict_in.items():
        if not val: ## if empty dictionary as value
            lg.functions.debug(f"making relative path: {key}")
            os.makedirs(key, exist_ok=True)

        else: ## append the filepath to all child nodes
            for subkey in list(val):
                new_subkey = f"{key}/{subkey}"
                lg.functions.debug(f"changing key {subkey} to {new_subkey}")
                val[new_subkey] = val.pop(subkey) ## replace with new subkey

    for key, val in dict_in.items():
        mkdirs_from_dict(val)

    return
