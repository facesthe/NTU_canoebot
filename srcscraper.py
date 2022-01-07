# new module that can view any bookable faclilty in SRC
import json as jsn

from pandas.io.formats.format import return_docstring
import Dotionary as Dot
import pandas as pd
import time
from datetime import date
from dateutil.parser import parse
import debuglogging as dl

log = dl.log
log.debug("srcscraper loaded")

_path = './srcscraper.config.json' ## path to srcscraper.config


## creating config variable in Dotionary form
with open(_path) as jsonfile:
    json = jsn.load(jsonfile)

config = [Dot for i in range(len(json))]

for i in range(len(config)):
    config[i] = Dot.to_dotionary(json[i])

# for item in config:
#     print(item)


def show_facility_table() -> pd.DataFrame:
    '''Returns a dataframe that contains facilities to be viewed'''

    returndf = pd.DataFrame({"no":[], "facility":[]})
    for i in range(len(config)):
        returndf = returndf.append(pd.DataFrame({"no":[i+1], "facility":[config[i].name]}))

    returndf = returndf.convert_dtypes() ## set to ints
    return returndf


def get_booking_table(tablecol:int, date:date) -> pd.DataFrame:
    '''Returns the booking table for a particular day.\n
    Raw table output, taken direct from page'''

    datereq = date.strftime('%d-%b-%y').upper()

    url = f"https://wis.ntu.edu.sg/pls/webexe88/srce_smain_s.srce$sel31_v?choice=1&fcode={config[tablecol].codename}&fcourt={config[tablecol].courts}&ftype=2&p_date={datereq}&p_mode=2"

    table = pd.read_html(url)[0]
    return table


def format_booking_table(table:pd.DataFrame, tablecol:int) -> pd.DataFrame:
    '''Correctly formats the raw table into summary table.\n
    Only data from the current day (third col) is taken.'''

    ## blank dataframe
    avail_df = pd.DataFrame({'time':[],'slots':[]})
    table = table.iloc[:, [0,2]]
    courts = config[tablecol].courts

    for multi in range(int(len(table)/courts)):
        ## hourly dataframe, each hour has $(courts) slots
        hour = table.iloc[multi*courts,0]
        data_hr = table.iloc[multi*courts:multi*courts+courts,[1]]

        try:
            count = data_hr.value_counts()['Avail']
        except:
            if (data_hr.value_counts()[0] == 20): ## if there ares 20 of the same entries
                count = data_hr.iloc[0,0]         ## take the name of that entry, e.g. NTU DBM
            else:
                count = 0

        avail_df = avail_df.append({'time':hour,'slots':count},ignore_index=True)

    ## final formatting step
    avail_df = avail_df.convert_dtypes() ## convert to respective data types
    avail_df = avail_df.astype(str) ## then convert all to strings


    return avail_df


# x = show_facility_table()
# print(x)
column = 9

y = get_booking_table(column, date.today())
# print(y.info())
print(y)
z = format_booking_table(y, column)
print(z)
