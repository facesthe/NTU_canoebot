import re
import json
import requests as rq
from lxml import html
import copy
from datetime import date, time

## template JSON for form information
sampleJSON = {
    "title": None,
    "description": None,
    "formID": None,
    "fields": None, ## None or list of samplefields
    "formfields": None
}

## template JSON for form field information
## optional as some fields are open-ended)
samplefield = {
    "name": None,
    "id": None,
    "idstr": None,
    "options": None ## None or list
}

'''Some pointers on using this module:
Dates are formatted in string form as YYYY-MM-DD.
Dates that are not filled in this format will cause a submission error.
All responses are in string form, responses that correspond to a
checkbox/selection must be in string form and match exactly.
For forms that contain required questions, all such fields must be populated with a response.'''
class gForm():
    '''Object that handles google form data

    Object can be initialised with either formID or filepath, but not both
    :param formID: Google form id
    :param filepath: Local formatted JSON (optional kwarg)
    '''
    def __init__(self, formID=None, filepath=None):
        self.title = None
        self.desc = None
        self.formID = None
        self.fields = None
        self.form = None ## fJSON formfields
        self.fJSON = None
        self.rawJSON = None

        ## construct all parameters if form id is passed as parameter
        if formID is not None:
            self.parse(formID)
        elif filepath is not None:
            self.import_json(filepath)

        return

    def __repr__(self):
        return f'Title: {self.title}\n'+\
            f'Description: {self.desc}\n'+\
            f'Form ID: {self.formID}\n'+\
            f'Fields: {[field["name"] for field in self.fields]}\n'+\
            f'Form Fields: {self.form}'

    def parse(self, formID):
        '''Primary constructor. Use if formID parameter not passed during object creation'''
        self.__get_raw_json(formID)
        self.__format_raw_json()
        self.__assign_attr()
        self.__generate_empty_form()

        return

    def import_json(self, filepath):
        '''Alternate constructor. Loads a local copy of form'''
        with open(filepath, 'r') as f:
            self.fJSON = json.loads(f.read())

        self.__assign_attr()

        return

    def to_raw_json(self):
        '''Returns the raw JSON taken from the form'''
        return self.rawJSON

    def to_formatted_json(self):
        '''Returns a formatted JSON of the form'''
        return self.fJSON

    def to_fstring(self):
        '''Returns a pretty-formatted JSON string'''
        return json.dumps(self.fJSON, indent=4)

    def export_raw_json(self, filepath):
        '''Write the raw json to file. Pretty unreadable ngl'''
        with open(filepath, 'w') as f:
            f.write(json.dumps(self.rawJSON))
        return

    def export_formatted_json(self, filepath):
        '''Write the formatted json to file'''
        with open(filepath, 'w') as f:
            f.write(json.dumps(self.fJSON))
        return

    def fill(self, fieldno:int, response:str):
        '''Fill the specified field with a string response'''
        self.form[self.fields[fieldno]['idstr']] = response
        self.__update_fJSON()
        return

    def fill_date(self, fieldno:int, date_in:date):
        '''Fill the specified field with a date'''
        self.form[self.fields[fieldno]['idstr']] = date_in.strftime('%Y-%m-%d')
        self.__update_fJSON()
        return

    def fill_time(self, fieldno:int, time_in:time):
        '''Fill the specified form with a time'''
        self.form[self.fields[fieldno]['idstr']] = time_in.strftime('%H:%M')
        self.__update_fJSON()
        return

    def fill_option(self, fieldno:int, optionno:int):
        '''Fill the specified field with a target option.
        Field must contain selectable options.'''

        ## check if there are fields present
        if self.fields[fieldno]['options'] is None:
            raise AttributeError(f'Field {fieldno} has no options available')

        ## check if index passed is within bounds
        elif optionno+1 > len(self.fields[fieldno]['options']):
            raise LookupError(f'Option number {optionno} out of range ({len(self.fields[fieldno]["options"])})')

        ## perform assignment
        else:
            self.form[self.fields[fieldno]['idstr']] = \
                self.fields[fieldno]['options'][optionno]

        self.__update_fJSON()
        return

    def submit(self):
        '''Submits the form. Returns 1 if successful, 0 if unsuccessful'''
        url = f'https://docs.google.com/forms/d/e/{self.formID}/formResponse'
        for key in self.fJSON['formfields'].keys():
            if self.fJSON['formfields'][key] is None:
                self.fJSON['formfields'][key] = ''

        response = rq.post(url, data = self.fJSON['formfields'])
        if (response.status_code == 200):
            return 1
        else:
            return 0

    def __get_raw_json(self, formID):
        '''Pulls the raw form data, with minimal formatting'''
        url = f'https://docs.google.com/forms/d/e/{formID}/viewform'
        response = rq.get(url)
        tree = html.fromstring(response.content)
        script = tree.xpath('//script[@type="text/javascript"]/text()') \
            ## successful in obtaining script section of html

        jsonstr = None ## init default

        for item in script:
            if 'var FB_PUBLIC_LOAD_DATA' in item:
                jsonstr = item
                break

        ## format json properly
        jsonstr = re.sub(r'^.*=\s', '', jsonstr) ## remove variable name
        jsonstr = re.sub(r';', '', jsonstr) ## remove semicolon

        self.rawJSON = json.loads(jsonstr)

        return

    def __format_raw_json(self):
        '''Formats form into human-readable JSON'''
        ## assigning to JSON
        rawfields = copy.deepcopy(self.rawJSON[1][1])
        returnJSON = sampleJSON
        returnJSON['title'] = self.rawJSON[3]
        returnJSON['description'] = self.rawJSON[1][0]
        returnJSON['formID'] = re.sub(r'^.*/', '', self.rawJSON[14])
        returnJSON['fields'] = [copy.deepcopy(samplefield) for i in range(len(rawfields))]

        ## assigning fields
        offset = 0
        for i in range(len(rawfields)):
            # print(rawfields[i][1])
            returnJSON['fields'][i+offset]['name'] = rawfields[i][1]

            try: ## attempt to get field id
                returnJSON['fields'][i+offset]['id'] = rawfields[i][4][0][0]
                returnJSON['fields'][i+offset]['idstr'] = f'entry.{rawfields[i][4][0][0]}'
            except: ## attempt failed, index does not correspond to a field (question), skip
                # print('indexing of None occured, skipping current index')
                returnJSON['fields'].pop(i+offset)
                offset -= 1
                continue

            # assigining field options (if any)
            if rawfields[i][4][0][1] is not None:
                returnJSON['fields'][i+offset]['options'] = \
                    [rawfields[i][4][0][1][j][0] for \
                    j in range(len(rawfields[i][4][0][1]))]

        self.fJSON = returnJSON
        return

    def __assign_attr(self):
        '''Assign main object parameters using formatted JSON data'''
        self.title = self.fJSON['title']
        self.desc = self.fJSON['description']
        self.formID = self.fJSON['formID']
        self.fields = self.fJSON['fields']
        self.form = self.fJSON['formfields']
        return

    def __update_fJSON(self):
        '''Update the form fields in fJSON from self.form'''
        for key, value in self.form.items():
            self.fJSON['formfields'][key] = value
        return

    def __generate_empty_form(self):
        '''Generate an empty dictionary that contains form responses.
        Requires that self.fields attribute is populated'''
        self.fJSON['formfields'] = {}

        for field in self.fields:
            self.fJSON['formfields'][field['idstr']] = None
        self.form = self.fJSON['formfields']
        return
