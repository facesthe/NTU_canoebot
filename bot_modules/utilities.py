'''Utilities module for bot_modules package'''

import functools

import telebot

VERIFY_MISSING_BEHAVIOR: bool = True
'''what to return if the lookup for verify_exec fails'''


def verify_exec(lookup: dict, key: str = None):
    '''Function decorator.
    Checks for matching key in dictionary,
    else checks for matching function name (capitalized).
    Executes the wrapped function if the condition is true.
    Does not execute otherwise.
    '''

    def inner(_function):
        # if key is None:
        #     key = str(_function.__name__).upper()

        @functools.wraps(_function)
        def wrapper(*args, **kwargs):

            inner_key = key
            if inner_key is None:
                inner_key = str(_function.__name__).upper()
            else:
                inner_key = str(key).upper()

            if inner_key in lookup.keys():
                result = lookup[inner_key]
            else:
                result = VERIFY_MISSING_BEHAVIOR

            if result == False:
                return lambda x: None ## do not exec
            else:
                return _function(*args, **kwargs)

        return wrapper
    return inner

def strip_message_command(msg: telebot.types.Message) -> str:
    '''Removes first word (the command) from message'''

    return ' '.join(msg.text.split()[1:])
