'''Interface to SCF for submitting daily log sheet'''

import requests as rq
from datetime import date, timedelta
from dateutil.parser import parse
import random

## bot modules
import lib.liblog as lg

import modules.sheetscraper as ss
import lib.gForm as gf
import modules.settings as s

lg.functions.debug("formfiller loaded")

## change the form id if needed (should be never)
## if SCF suddenly decides to change their form, then formfiller will need to be updated
FORM_ID:str = s.json.formfiller.form_id

## if the exco changes then pls fill in new details
## dictionary is in this format:
## key:     [str your_name]
## value:   [str your_8_digit_hp_number]
PARTICULARS:dict = s.json.formfiller.particulars

## start, end time constants
## change these in the config file if necessary
AM_START_TIME   :str = s.json.formfiller.times.am.start
AM_END_TIME     :str = s.json.formfiller.times.am.end
PM_START_TIME   :str = s.json.formfiller.times.pm.start
PM_END_TIME     :str = s.json.formfiller.times.pm.end

## previously submitted dates for each timeslot (curr 2)
SUBMITTED_TIMES: "list[date]" = [date.today() - timedelta(days=1) for i in range(2)]
'''previously submitted dates for each timeslot (curr 2)'''

url = f'https://docs.google.com/forms/d/e/{FORM_ID}/formResponse'

def is_submitted_before(date_in: date, time_slot: int) -> bool:
    '''Checks if logsheet was submitted before. Updates global if not submitted before'''
    global SUBMITTED_TIMES

    if SUBMITTED_TIMES[time_slot] != date_in:
        SUBMITTED_TIMES[time_slot] = date_in
        return False
    else:
        return True

class logSheet():
    def __init__(self):
        self.name       = None  ## key in the particulars dictionary
        self.contact    = None  ## value in the particulars dictionary
        self.star1      = None  ## number of 1 star people
        self.star0      = None  ## number of 0 star people
        self.date       = None  ## date in datetime.date format
        self.datestr    = None  ## date in string form
        self.timeslot   = 0     ## AM or PM timeslot, enum(0,1), default 0
        self.isoday     = None  ## day represented as a number
        self.ispresent  = None  ## if log sheet day is same as current day
        self.starttime  = None  ## earliest-boat-in-water time (estimate)
        self.endtime    = None  ## latest-boat-out-of-water time (also estimate)
        self.form       = None  ## Constructed form
        self.gForm      = gf.gForm(FORM_ID)

    ## modifies name and contact
    def getparticulars(self):
        self.name,self.contact = random.choice(list(PARTICULARS.items()))

    def __certstatus(self):
        return ss.CERT_STATUS

    def __getnamelist(self):
        timeslot = 'am' if self.timeslot==0 else 'pm'
        df = ss.namelist(f'{self.datestr}, {timeslot}')[3:]
        return df.reset_index(drop=True)

    ## modifies star1 and star0
    def getcertstatus(self):
        df = self.__getnamelist()
        count = len(df)
        certstatus = self.__certstatus() ## get the dictionary

        for i in range(count):
            df.iloc[i] = certstatus[df.iloc[i]] ## replace with 1s and 0s

        self.star1 = df.sum()
        self.star0 = count - self.star1

    ## modifies date, datestr, ispresent and isoday
    def getdate(self, date_in):
        try:
            d = parse(date_in).date()
        except:
            d = date.today()

        if d != date.today():
            self.ispresent = 0
        else:
            self.ispresent = 1

        self.date = d
        self.datestr = d.strftime('%Y-%m-%d')
        self.isoday = d.isoweekday()

    ## tells formfiller which time slot to pull through sheetscraper
    def settimeslot(self, timeslot:int):
        self.timeslot = timeslot

    ## modifies starttime and endtime
    ## no logic added yet
    def gettimes(self):
        if self.timeslot == 0:
            self.starttime = AM_START_TIME
            self.endtime = AM_END_TIME

        elif self.timeslot == 1:
            self.starttime = PM_START_TIME
            self.endtime = PM_END_TIME

    ## Change the number of people present for a session
    def changeattendance(self, new_count):
        self.star1 = new_count
        self.star0 = 0

    ## Change the name
    def changename(self, new_name):
        self.name = new_name

    def generateform(self, date_str=''):
        self.getdate(date_str)
        self.getparticulars()
        self.getcertstatus()
        self.gettimes()

        ## gForm used to fill form, removes the need to inspect element to get entry id
        self.gForm.fill_str(0, self.name)
        self.gForm.fill_str(1, self.contact)
        self.gForm.fill_str(2, "Nanyang Technological University")
        self.gForm.fill_option(3, 1)
        self.gForm.fill_int(4, self.star1)
        self.gForm.fill_int(5, self.star0)
        self.gForm.fill_option(6, 1)
        self.gForm.fill_date(7, self.date)
        self.gForm.fill_str(8, self.starttime)
        self.gForm.fill_str(9, self.endtime)
        self.gForm.fill_option(10, 0)

    def submitform(self):
        return self.gForm.submit()
