import re
from gForm import gForm
from datetime import date, time

# bikex_form = gForm('1FAIpQLScblJJuHMBwCx-iK230jlxNxCVOPOauHjkQLMBkhdH1KG9OBA')
test_form = gForm('1FAIpQLSd1v5K-2xVXX5VTq_7sWROBZhKfeaeypQUnHKkfiBmweBDqBQ')


test_form.fill(1, '13:23')
result = test_form.submit()
print(test_form)
print(result)
# print(test_form.to_fstring())