import pandas as pd
import formscraper
import json

jsonobj = formscraper.get_raw_json('1FAIpQLSefOwdgq9W4HJGNy2aa8q5oRnj3wHx4mOtGHcE7yhIkM_x2Pw') ## ITCC

# jsonobj = formscraper.get_raw_json('1FAIpQLScblJJuHMBwCx-iK230jlxNxCVOPOauHjkQLMBkhdH1KG9OBA') ## hall X


jsonformatted = formscraper.format_raw_form(jsonobj)

with open('itcc-availability-newformatted.json','w') as f:
    f.write(json.dumps(jsonformatted))

