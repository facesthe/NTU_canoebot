'''testing prototype formscraper module'''

import requests as rq
import json
import xmltojson
from lxml import html
import re

# debug form id: 1uXwIqNcDpnyZYAL1uH5D8k-qZSAqcfrIVY6yw5kOyII
# url = f'https://docs.google.com/forms/d/e/{FORM_ID}/formResponse'

import TrainingLog
log = TrainingLog.TrainingLog()
log.fill_name('testing name')
log.fill_date('2 feb 2022')
log.fill_sleephr(10)
log.fill_rhr(60)
log.fill_comments('test comment testing')

log.fill_form()

status = log.submit_form()
if status == 1:
    print('submission success')
else:
    print('submission failed')

