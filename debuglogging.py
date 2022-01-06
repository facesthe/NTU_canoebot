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


logging.basicConfig(stream = sys.stdout, level = loglevel)
log = logging.getLogger()
'''logger class\n
Use: log.info("statement"), log.debug("statement")\n
Use f-strings for formatting'''
