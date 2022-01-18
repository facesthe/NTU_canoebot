import re
import json
import requests as rq
from lxml import html
import copy

## template JSON for form information
sampleJSON = {
    "title": None,
    "description": None,
    "formid": None,
    "fields": None, ## None or list of samplefields
}

## template JSON for form field information
## optional as some fields are open-ended)
samplefield = {
    "name": None,
    "id": None,
    "options": None ## None or list
}

class gForm():

    def __init__(self, form_id=None):
        self.title = None
        self.desc = None
        self.formId = None
        self.fields = None
        self.rawJSON = None
        self.fJSON = None

        ## construct all parameters if form id is passed as parameter
        if form_id is not None:
            self.parse(form_id)


    def parse(self, form_id):
        '''Constructor. Use if form_id parameter not passed during object creation'''
        self.__get_raw_json(form_id)
        self.__format_raw_json()

        self.title = self.fJSON['title']
        self.desc = self.fJSON['description']
        self.formId = self.fJSON['formid']
        self.fields = self.fJSON['fields']

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

    def __get_raw_json(self, form_id):
        '''Pulls the raw form data, with minimal formatting'''
        url = f'https://docs.google.com/forms/d/e/{form_id}/viewform'
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
        returnJSON['formid'] = re.sub(r'^.*/', '', self.rawJSON[14])
        returnJSON['fields'] = [copy.deepcopy(samplefield) for i in range(len(rawfields))]

        ## assigning fields
        offset = 0
        for i in range(len(rawfields)):
            # print(rawfields[i][1])
            returnJSON['fields'][i+offset]['name'] = rawfields[i][1]

            try: ## attempt to get field id
                returnJSON['fields'][i+offset]['id'] = rawfields[i][4][0][0]
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


