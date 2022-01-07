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


def parse_date(date_str:str) -> date:
    '''Parse input string into a date object.\n
    Invalid dates are interpreted as the current day'''
    try:
        date_obj = parse(date_str)
    except:
        date_obj = date.today()

    return date_obj


def show_facility_table() -> pd.DataFrame:
    '''Returns a dataframe that contains facilities to be viewed'''

    returndf = pd.DataFrame({"no":[], "facility":[]})
    for i in range(len(config)):
        returndf = returndf.append(pd.DataFrame({"no":[i+1], "facility":[config[i].name]}))

    returndf = returndf.convert_dtypes() ## set to ints
    return returndf.to_string(index=False)


def get_booking_table(date:date, tablecol:int) -> pd.DataFrame:
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
            if (data_hr.value_counts()[0] == courts):   ## if all entries are the same
                if courts == 1: ## for privacy reasons
                    count = 0
                else:
                    count = data_hr.iloc[0,0]           ## take the name of that entry, e.g. NTU DBM
            else:
                count = 0

        avail_df = avail_df.append({'time':hour,'slots':count},ignore_index=True)

    ## final formatting step
    avail_df = avail_df.convert_dtypes() ## convert to respective data types
    avail_df = avail_df.astype(str) ## then convert all to strings
    avail_df.iloc[:,1] = avail_df.iloc[:,1].apply\
        (lambda x: x + f"/{courts}" if x.isnumeric() else x)

    return avail_df


def get_booking_result(date:date, tablecol:int) -> str:

    ## fetch and format block
    t_start = time.time()
    table = get_booking_table(date, tablecol)
    t_end = time.time()
    table_f = format_booking_table(table, tablecol)

    exec_time = "{:5.4f}".format(t_end - t_start) ##in seconds, to check on fetch speed

    ## constructing string
    returnstr = f"{date.strftime('%d %b %y, %A')}\n{config[tablecol].name}\n\n{table_f.to_string(index=False)}\n\nfetch time: {exec_time}s"

    return returnstr



# x = show_facility_table()
# print(x)
# column = 13

# y = get_booking_table(date.today(), column)
# # print(y.info())
# print(y)
# z = format_booking_table(y, column)
# print(z)

# a = get_booking_result(date.today(), 14)
# print(a)