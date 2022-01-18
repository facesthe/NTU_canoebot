from gForm import gForm

bikex_form = gForm('1FAIpQLScblJJuHMBwCx-iK230jlxNxCVOPOauHjkQLMBkhdH1KG9OBA')

print(bikex_form)
bikex_form.fill(0, 'Terence Howard')
bikex_form.fill(1, 'SCSE/2')
bikex_form.fill_option(2, 0)
# bikex_form.fill_from_option(3, 2)
# bikex_form.export_formatted_json('formscraper/bikex-form.json')

print(bikex_form.to_fstring())

# testlog = gForm(filepath='formscraper/testlog-form.json')
# testlog.fill(0, '01-10-2022')
# testlog.fill(1, 'Mickey Mouse')
# testlog.fill(2, '10')
# testlog.fill(0, '2022-01-16')
# testlog.fill(3, '55')
# testlog.fill(4, 'auto submission 3')
# print(testlog)
# testlog.export_formatted_json('formscraper/testlog-form.json')

# print(testlog.to_fstring())
# status = testlog.submit()
# print(status)
# testlog.export_formatted_json('formscraper/testlog-form.json')