import requests as rq
import re
from lxml import html
import json


def get_raw_form_json(form_id:str):
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

