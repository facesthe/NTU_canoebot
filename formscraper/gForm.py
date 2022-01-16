import re
import json
import requests as rq
from lxml import html

class gForm():
    def __init__(self, form_id=None):
        self.title = None
        self.formId = None
        self.desc = None
        self.fields = None
        self.rawJSON = None

        ## construct all parameters if form id is passed as parameter
        if form_id is not None:
            self.parse(form_id)


    def parse(self, form_id):
        return

    def to_json(self):
        return

    def __get_raw_json(self, form_id):
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



