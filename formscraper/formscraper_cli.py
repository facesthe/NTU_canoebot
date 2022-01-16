import requests as rq
import re
from lxml import html

form_id = input('Paste the form id here: ')
file_name = input('Enter the outfile name: ')

url = f'https://docs.google.com/forms/d/e/{form_id}/viewform'

print(f'requesting data from {url} ...')
response = rq.get(url)
tree = html.fromstring(response.content)

script = tree.xpath('//script[@type="text/javascript"]/text()') ## successful in obtaining script section of html

jsonstr = None

for item in script:
    if 'var FB_PUBLIC_LOAD_DATA' in item:
        # print(item)
        jsonstr = item
        break

if jsonstr == None:
    print('No form data found.')
    exit(-1)

## format into a json string
jsonstr = re.sub(r'^.*=\s', '', jsonstr)
jsonstr = re.sub(r';', '', jsonstr)

with open(f'{file_name}.json', 'w') as f:
    f.write(jsonstr)
print(f'form fields written to {file_name}.json')
