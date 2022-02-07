'''testing prototype formscraper module'''

import requests as rq
import json
import xmltojson
from lxml import html
import re
from datetime import date

# debug form id: 1uXwIqNcDpnyZYAL1uH5D8k-qZSAqcfrIVY6yw5kOyII
# url = f'https://docs.google.com/forms/d/e/{FORM_ID}/formResponse'

import TrainingLog
log = TrainingLog.TrainingLog()
log.fill_name('telegram username') ## name filled using Telegram first name (but can be edited)
log.fill_date(date(2022, 2, 2))
log.fill_sleephr(6)
log.fill_rhr(100)
log.fill_comments('training was shite')

log.fill_form()

status = log.submit_form()
if status == 1:
    print('submission success')
else:
    print('submission failed')

