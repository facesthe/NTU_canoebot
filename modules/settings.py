'''Settings importer'''

import os
import tomllib

import lib.Dotionary as dot


_path_prefix = "./.configs"

# these paths are hardcoded for now
_paths: dict = {
    "deploy": "botsettings.template.deploy.toml",
    "debug": "botsettings.template.debug.toml"
}

_template_path: str = "botsettings.template.toml"

json: dot.Dotionary
'''Settings are stored in this variable'''

def recursive_update(template: dict, compare: dict):
    '''
    Recursively updates a dictionary against a template.
    Adds key-value pairs if they don't exist.'''
    for key in template.keys():
        if key not in compare:
            compare[key] = template[key]

        if type(template[key]) == dict:
            recursive_update(template[key], compare[key])

def recursive_reorder(template: dict, compare: dict, result: dict):
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
    """Generate settings from config files and place the result in global var `json`
    """
    global json

    _settings: dict = {}

    # read in private settings
    for key in _paths.keys():
        path = _paths[key]
        full_path = os.path.join(_path_prefix, path)

        if os.path.exists(full_path):
            with open(full_path, "rb") as tomlfile:
                _settings[key] = tomllib.load(tomlfile)

    _actual_settings: dict
    _has_a_valid_setting: bool = False

    # Takes the first setting variant that has use=True set.
    # if both have the key set, deploy is used.
    for key in _settings.keys():
        if _settings[key]["use"]:
            _actual_settings = _settings[key]
            _has_a_valid_setting = True
            break

    if not _has_a_valid_setting:
        raise Exception("Settings misconfigured. Check if toml files exist or that the KV 'use=true' has been set.")

    template: dict

    # read in public settings
    template_path = os.path.join(_path_prefix, _template_path)
    with open(template_path, "rb") as tomlfile:
        template = tomllib.load(tomlfile)

    merged_settings: dict = {}

    recursive_update(template, _actual_settings)
    recursive_reorder(template, _actual_settings, merged_settings)

    json = dot.to_dotionary(merged_settings)
