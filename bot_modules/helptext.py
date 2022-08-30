'''
The help module provides function decorators that
registers functions and their docstrings automatically.
These functions are used to show
This helps reduce duplication of comments/helptexts and cuts down
on wasted time.
'''

import functools

global HELP_TEXT_HANDLERS
global PUBLIC_HANDLERS
global PRIVATE_HANDLERS
global KNOWN_HANDLERS

HELP_TEXT_HANDLERS: dict = dict()
'''Contains all registered message handlers/functions'''

PUBLIC_HANDLERS: dict = dict()
'''Contains all public handlers available for use'''

PRIVATE_HANDLERS: dict = dict()
'''Contains all private handlers'''

KNOWN_HANDLERS: dict = dict()
'''Placeholder, for future use'''


def register_function(
    name: str,
    help: bool = False,
    public: bool = False,
    private: bool = False):
    '''Registers the decorated function by its corresponding command
    and its docstring (in the order that functions are defined).
    Specify where to register these functions to.
    '''

    def inner(_function):
        global \
            HELP_TEXT_HANDLERS,\
            PUBLIC_HANDLERS, \
            PRIVATE_HANDLERS,\
            KNOWN_HANDLERS

        if help:
            HELP_TEXT_HANDLERS[name] = _function.__doc__
        if public:
            PUBLIC_HANDLERS[name] = _function.__doc__
        if private:
            PRIVATE_HANDLERS[name] = _function.__doc__

        @functools.wraps(_function)
        def wrapper(*args, **kwargs):
            return _function(*args, **kwargs)

        return wrapper
    return inner
