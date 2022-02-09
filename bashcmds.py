## for sending bash commands to the bot
import subprocess as sp
# import debuglogging as dl
from lib.liblog import loggers as lg

# log = dl.log
lg.functions.debug("bashcmds loaded")

def bashout(bash_cmd):
    bash_out = sp.run(bash_cmd.split(), capture_output=True, text=True)
    return bash_out.stdout

def bashreply(str_in):
    return f'bash says:\n{str_in}'

def uptime():
    return bashreply(bashout("uptime -p"))

def echo(str_in =''):
    '''Echo: repeats whatever is passed to the function'''
    return bashreply(bashout(f"echo {str_in}"))

def botlog():
    '''Uses the bash alias canoebotlog to retrieve most recent logs'''
    return bashreply(bashout("sudo bash ./.canoebotlog.sh"))