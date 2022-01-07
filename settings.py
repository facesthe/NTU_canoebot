'''Settings for canoebot.
To modify settings see botsettings.json'''
import json as jsn
import Dotionary as dot

## path to settings file - do not change!
## modify the json file to change settings
_path = './botsettings.debug.json'

with open(_path) as jsonfile:
    json = jsn.load(jsonfile)

json = dot.to_dotionary(json)
