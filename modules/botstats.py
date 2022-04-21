from datetime import date

class BotStats():
    '''A stat counter for tracking bot interactions\n
    Still building'''
    def __init__(self):
        self.botcount = 0
        self.botcalls = 0
        self.charcount = 0
        self.date = date.today()

    def resetstats(self):
        '''reset all stat counters'''
        self.__init__()
    
    def update(self, message_text):
        '''update the bot stats'''
        self.botcalls += 1
        self.charcount += len(message_text)
        if 'bot' in message_text.lower():
            self.botcount += 1

    def parse(self, message_text):
        '''parse through message and resets counters by date'''
        if self.date != date.today():
            self.resetstats()
            self.date = date.today()
        self.update(message_text)