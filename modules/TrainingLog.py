'''Object containing training log information.\n
Passed as a parameter between telebot functions.'''

from datetime import date
from time import sleep
from dateutil.parser import parse
from lib.gForm import gForm
import json as jsn

import modules.settings as s


class TrainingLog():
    '''Wrapper class for gForm.\n
    Note that all input validation should be done using separate methods'''
    def __init__(self):
        self.name = None
        self.date = None
        self.sleephr = None
        self.energy = None
        self.rhr = None
        self.miles = None
        self.comments = None
        self.gForm = gForm(s.json.traininglog.form_id)

    def __repr__(self):
        return jsn.dumps(
            {
                "name":self.name,
                "date":str(self.date),
                "sleephr":self.sleephr,
                "energy": self.energy,
                "rhr":self.rhr,
                "mileage":str(self.miles),
                "comments":self.comments
            }, indent=2
        )

    def fill_name(self, name_in:str):
        self.name = name_in

    def parse_json_data(self, json_in:str):
        json = jsn.loads(json_in)
        self.name = json["name"]
        self.date = self.dateparser(json["date"])
        self.sleephr = json["sleephr"]
        self.energy = json["energy"]
        self.rhr = json["rhr"]
        self.miles = float(json["mileage"])
        self.comments = json["comments"]

    def fill_date(self, date_in:date):
        self.date = date_in
        return

    def fill_sleephr(self, sleephr_in:int):
        self.sleephr = sleephr_in
        return

    def fill_energy(self, energy_in:int):
        self.energy = energy_in
        return

    def fill_rhr(self, rhr_in:int):
        self.rhr = rhr_in
        return

    def fill_mileage(self, miles:float):
        self.miles = miles
        return

    def fill_comments(self, comments_in:str):
        self.comments = comments_in
        return

    def fill_form(self):
        self.gForm.fill_str(0, self.name)
        self.gForm.fill_date(1, self.date)
        self.gForm.fill_int(2, self.sleephr)
        self.gForm.fill_int(3, self.energy)
        self.gForm.fill_int(4, self.rhr)
        self.gForm.fill_float(5, self.miles) ## new insertion
        self.gForm.fill_str(6, self.comments)
        return

    def submit_form(self):
        return self.gForm.submit()

    @staticmethod
    def dateparser(str_in)->date:
        '''Internal date parser'''
        try:
            return parse(str_in).date()
        except:
            return date.today()
