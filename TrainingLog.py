'''Object containing training log information.\n
Passed as a parameter between telebot functions.'''

from datetime import date
from time import sleep
from dateutil.parser import parse
from formscraper.gForm import gForm
import settings as s


class TrainingLog():
    '''Wrapper class for gForm.\n
    Note that all input validation should be done outside of this class'''
    def __init__(self):
        self.name = None
        self.date = None
        self.sleephr = None
        self.rhr = None
        self.comments = None
        self.gForm = gForm(s.json.traininglog.form_id)

    def fill_name(self, name_in:str):
        self.name = name_in
        return

    def fill_date(self, date_in:date):
        self.date = date_in
        return

    def fill_sleephr(self, sleephr_in:int):
        self.sleephr = sleephr_in
        return

    def fill_rhr(self, rhr_in:int):
        self.rhr = rhr_in
        return

    def fill_comments(self, comments_in:str):
        self.comments = comments_in
        return

    def fill_form(self):
        self.gForm.fill_str(0, self.name)
        self.gForm.fill_date(1, self.date)
        self.gForm.fill_int(2, self.sleephr)
        self.gForm.fill_int(3, self.rhr)
        self.gForm.fill_str(4, self.comments)
        return

    def submit_form(self):
        return self.gForm.submit()
