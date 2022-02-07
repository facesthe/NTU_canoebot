'''I've decided to put the logging settings here so that all modules can call this\n
Debug level globally modified over here'''
import logging, sys
import settings as s

levels = {
    '''Dictionary containing the level of logging - default INFO'''
    'CRITICAL'  : 50,
    'ERROR'     : 40,
    'WARNING'   : 30,
    'INFO'      : 20, ## sees everything at or above info (using this mostly)
    'DEBUG'     : 10, ## sees everything at or above debug (for debugging, duh)
    'NOTSET'    : 0
}

## modify the botsettings.json file to change log settings
loglevel = s.json.logger.debug_level

logging.basicConfig(
    stream = sys.stdout,
    level = loglevel,
    format='%(levelname)-8s %(asctime)s {%(module)s}:[%(funcName)s] %(message)s',
    datefmt='%Y-%m-%d %H:%M:%S'
    )

log = logging.getLogger()
'''logger class\n
Use: log.info("statement"), log.debug("statement")\n
Use f-strings for formatting'''

def log_notset():
    return

def log_debug(function):
    '''Log decorator for function/method calls'''
    def wrapper(*args, **kwargs):
        log.debug('%(levelname)-8s %(asctime)s')
        return function(*args, **kwargs)

    return wrapper

def log_info():
    return

def log_warning():
    return

def log_error():
    return

def log_critical():
    return

