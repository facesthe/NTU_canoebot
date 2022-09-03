'''Utilities module for bot_modules package'''

import functools

VERIFY_MISSING_BEHAVIOR: bool = True
'''what to return if the lookup for verify_exec fails'''


def verify_exec(lookup: dict, key: str):
    '''Function decorator.
    Checks for a corresponding entry in a dictionary,
    key corresponds to the function's capitalized name.
    Executes the wrapped function if the condition is true.
    Does not execute otherwise
    '''

    def inner(_function):
        # func_name: str = _function.__name__
        # func_name = func_name.upper()
        @functools.wraps(_function)
        def wrapper(*args, **kwargs):
            try:
                result = lookup[key]
            except:
                result = VERIFY_MISSING_BEHAVIOR

            if result == False:
                return lambda x: None ## do not exec
            else:
                return _function(*args, **kwargs)

        return wrapper
    return inner
