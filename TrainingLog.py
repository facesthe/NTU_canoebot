'''Object containing training log information.\n
Passed as a parameter between telebot functions.'''

# debug form id: 1uXwIqNcDpnyZYAL1uH5D8k-qZSAqcfrIVY6yw5kOyII
# url = f'https://docs.google.com/forms/d/e/{FORM_ID}/formResponse'
from datetime import date
from dateutil.parser import parse

class TrainingLog():
    def __init__(self):
        self.date = None
        self.datestr = None
        self.name = None
        self.sleephr = None
        self.rhr = None

