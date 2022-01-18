from gForm import gForm

itcc_form = gForm('1FAIpQLSefOwdgq9W4HJGNy2aa8q5oRnj3wHx4mOtGHcE7yhIkM_x2Pw')
# form = gForm(filepath='hall-10-bikex-newformatted.json')
form = gForm(formID='1FAIpQLScblJJuHMBwCx-iK230jlxNxCVOPOauHjkQLMBkhdH1KG9OBA')
ssm_wavier = gForm(formID='1FAIpQLSflTex9nwJmz-SuTWTAf5TjTdjPnRrTYSMieMkpOVorbIaqhw')

form.export_formatted_json('formscraper/hall-10-bikex-newformatted.json')
itcc_form.export_formatted_json('formscraper/itcc-newformatted.json')
print(form)
print(list(form.formfields.keys()))
# print(form.desc)