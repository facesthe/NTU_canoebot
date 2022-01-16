'''testing prototype formscraper module'''

import requests as rq
import json
import xmltojson
from lxml import html
import re

# debug form id: 1uXwIqNcDpnyZYAL1uH5D8k-qZSAqcfrIVY6yw5kOyII
# url = f'https://docs.google.com/forms/d/e/{FORM_ID}/formResponse'

url2 = "https://docs.google.com/forms/d/e/1FAIpQLSflTex9nwJmz-SuTWTAf5TjTdjPnRrTYSMieMkpOVorbIaqhw/viewform"
url = "https://docs.google.com/forms/d/e/1FAIpQLSd1v5K-2xVXX5VTq_7sWROBZhKfeaeypQUnHKkfiBmweBDqBQ/viewform"

response = rq.get(url)
# json = xmltojson.parse(response.text)
# print(json)
# print(response.text)
tree = html.fromstring(response.content)
script = tree.xpath('//script[@type="text/javascript"]/text()') ## successful in obtaining script section of html
for item in script:
    if 'var FB_PUBLIC_LOAD_DATA' in item:
        # print(item)
        jsonstr = item
        break

## format into a json string
jsonstr = re.sub(r'^.*=\s', '', jsonstr)
jsonstr = re.sub(r';', '', jsonstr)
print(jsonstr)

jsonobj = json.loads(jsonstr)
print(jsonobj)

with open('scraped-sheet-data.json-1', 'w') as f:
    f.write(json.dumps(jsonobj))

# values = re.findall(r'var.*?=\s*(.*?);', script[1], re.DOTALL | re.MULTILINE)
# for value in values:
#     print(json.loads(value))
# jsonfile = json.loads(values[0])
# print(json.dumps(jsonfile, indent=2, sort_keys=True)) ## successful in obtaining json-ified data

