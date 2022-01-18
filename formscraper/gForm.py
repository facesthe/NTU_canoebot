import re
import json
import requests as rq
from lxml import html
import copy

## template JSON for form information
sampleJSON = {
    "title": None,
    "description": None,
    "formID": None,
    "fields": None, ## None or list of samplefields
}

## template JSON for form field information
## optional as some fields are open-ended)
samplefield = {
    "name": None,
    "id": None,
    "idstr": None,
    "options": None ## None or list
}

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
        self.fJSON = None
        self.rawJSON = None
        self.formfields = None

        ## construct all parameters if form id is passed as parameter
        if formID is not None:
            self.parse(formID)
        elif filepath is not None:
            self.load_json(filepath)

        return

    def __repr__(self):
        return f'Title: {self.title}\n'+\
            f'Description: {self.desc}\n'+\
            f'Form ID: {self.formID}\n'+\
            f'Form Fields: {self.formfields}\n'+\
            f'Fields: {[field["name"] for field in self.fields]}'

    def parse(self, formID):
        '''Constructor. Use if formID parameter not passed during object creation'''
        self.__get_raw_json(formID)
        self.__format_raw_json()
        self.__assign_attr()
        self.__generate_form()

        return

    def load_json(self, filepath):
        '''Load a local copy of form'''
        with open(filepath, 'r') as f:
            self.fJSON = json.loads(f.read())

        self.__assign_attr()
        self.__generate_form()

        return


    def to_raw_json(self):
        '''Returns the raw JSON taken from the form'''
        return self.rawJSON

    def to_formatted_json(self):
        '''Returns a formatted JSON of the form'''
        return self.fJSON

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
                # print(item)
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
        self.title = self.fJSON['title']
        self.desc = self.fJSON['description']
        self.formID = self.fJSON['formID']
        self.fields = self.fJSON['fields']
        return

    def __generate_form(self):
        '''Generate an empty dictionary that contains form responses.
        Requires that self.fields attribute is populated'''
        self.formfields = {}

        for field in self.fields:
            self.formfields[field['idstr']] = None
        return
