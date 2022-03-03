import re
from gForm import gForm
from datetime import date, time

# bikex_form = gForm('1FAIpQLScblJJuHMBwCx-iK230jlxNxCVOPOauHjkQLMBkhdH1KG9OBA')
# test_form = gForm('1FAIpQLSd1v5K-2xVXX5VTq_7sWROBZhKfeaeypQUnHKkfiBmweBDqBQ')

# # test_form.fill(1, '13:23')
# test_form.fill_time(1, time(23, 12))
# result = test_form.submit()
# print(test_form)
# print(result)
# print(test_form.to_fstring())

logsheet_form = gForm('1FAIpQLSfMtt0kvol72F9A2BaLJacr8Xzm9n51KBxVfS8YkDe8SfS5GA')
# print(logsheet_form.to_fstring())
# print(logsheet_form)
logsheet_form.export_formatted_json('./formscraper/logsheet_form.json')
logsheet_form.export_raw_json('./formscraper/logsheet_form_raw.json')
# print(logsheet_form.to_rstring())

# logsheet2_form = gForm('1FAIpQLSc1notGcZvtmSjFBDXxk_DZ4Q4_mRSAS4K6OlRUwRyg_o_c7g')
# print(logsheet2_form.to_fstring())
# print(logsheet2_form.to_rstring())