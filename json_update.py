'''
Run this script to regenerate botsettings[.debug].json \
from botsettings.template.json.
Attributes that are previously not present in the 2 template files \
will be updated off the main template json
To use:
    run "python3 json_update.py"
    or "import json_update"
'''

import json as jsn

## Change all common settings params here
TEMPLATE_PATH = '.configs/botsettings.template.json'

## Change setting-specific params here (unique to each file)
## paths for each index should correspond to the same settings type
## E.g. index 0: deployed settings file
SETTINGS_PATHS = [
    '.configs/botsettings.template.deploy.json',
    '.configs/botsettings.template.debug.json'
    ]
WRITEBACK_PATHS = [
    '.configs/botsettings.json',
    '.configs/botsettings.debug.json'
]


def read_json(path)->dict:
    '''Reads in JSON file as a dictionary object'''
    with open(path) as jsonfile:
        json = jsn.load(jsonfile)

    return json


def write_json(path, json):
    '''Writes a dictionary object as a pretty-printed JSON'''
    with open(path, 'w') as jsonfile:
        jsn.dump(json, jsonfile, indent=4)

    return


def recursive_update(template, compare):
    '''
    Recursively updates a dictionary against a template.
    Adds key-value pairs if they don't exist.'''
    for key in template.keys():
        if key not in compare:
            compare[key] = template[key]

        if type(template[key]) == dict:
            recursive_update(template[key], compare[key])


def recursive_reorder(template, compare, result):
    '''
    Takes the values behind compare and key order from template
    to construct a new dictionary following the order of template.
    At the minimum, 'template' needs to have an identical keyset to 'compare'.
    Additionally, 'template' can be a subset of 'compare' in terms of keys.
    If 'compare' contains additional keys not found in 'template',
    keys will be added after the matching process.
    'result' is a blank dictionary.'''

    ## order key-val pairs
    for key in template.keys():
        result[key] = compare[key]
        ## recurse
        if type(template[key]) == dict:
            result[key] = {}
            recursive_reorder(template[key], compare[key], result[key])

    ## add extra key-val pairs
    for key in compare.keys():
        if key not in template:
            result[key] = compare[key]

    return


def run():
    print("Constructing json settings files...")

    ## read in files
    template = read_json(TEMPLATE_PATH)
    compare = [{} for i in range(len(SETTINGS_PATHS))]
    result = [{} for i in range(len(SETTINGS_PATHS))]

    for i in range(len(SETTINGS_PATHS)):
        compare[i] = read_json(SETTINGS_PATHS[i])
        recursive_update(template, compare[i])
        recursive_reorder(template, compare[i], result[i])
        write_json(WRITEBACK_PATHS[i], result[i])

    print("json writeback done.")
    return
