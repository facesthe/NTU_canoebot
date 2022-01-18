import requests as rq
import re
from lxml import html
import json
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


def get_raw_json(form_id):
    '''Produces the raw script json from a specified form.\n
    Does not perform any manipulation on the fetched data'''

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

    return json.loads(jsonstr)


def format_raw_form(jsonobj):
    ## assigning to JSON
    rawfields = copy.deepcopy(jsonobj[1][1])
    returnJSON = sampleJSON
    returnJSON['title'] = jsonobj[3]
    returnJSON['description'] = jsonobj[1][0]
    returnJSON['formid'] = re.sub(r'^.*/', '', jsonobj[14])
    returnJSON['fields'] = [copy.deepcopy(samplefield) for i in range(len(rawfields))]

    ## assigning fields
    offset = 0
    for i in range(len(rawfields)):
        print(rawfields[i][1])
        returnJSON['fields'][i+offset]['name'] = rawfields[i][1]


        try: ## attempt to get field id
            returnJSON['fields'][i+offset]['id'] = rawfields[i][4][0][0]
        except: ## attempt failed, index does not correspond to a field (question), skip
            print('indexing of None occured, skipping current index')
            returnJSON['fields'].pop(i+offset)
            offset -= 1
            continue

        # print(rawfields[i][4][0][0])
        # print(returnJSON['fields'][i]['id'])

        # assigining field options (if any)
        if rawfields[i][4][0][1] is not None:
            returnJSON['fields'][i+offset]['options'] = \
                [rawfields[i][4][0][1][j][0] for \
                j in range(len(rawfields[i][4][0][1]))]



    return returnJSON
