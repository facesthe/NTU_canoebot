## for sending bash commands to the bot
import subprocess as sp
import lib.liblog as lg

lg.functions.debug("bashcmds loaded")

def bashout(bash_cmd):
    '''
    Evaluates and executes code input into the function.
    Note: can be used for malicious purposes
    '''
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
    '''Retrieve the local log file'''
    return bashreply(bashout("tail --bytes=1024 .scripts/canoebot.log"))
