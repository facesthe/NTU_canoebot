import json as jsn
import pandas as pd
import time
from datetime import datetime, date, timedelta
from dateutil.parser import parse
import copy

import lib.liblog as lg
import lib.Dotionary as Dot

lg.functions.debug("srcscraper loaded")

_path = './.configs/srcscraper.config.json' ## path to srcscraper.config

## creating config variable in Dotionary form
with open(_path) as jsonfile:
    json = jsn.load(jsonfile)

FACILITY_TABLE = [Dot for i in range(len(json))]
'''Contains SRC facility info in an array of dot-notation accessible dictionaries.'''

for i in range(len(FACILITY_TABLE)):
    FACILITY_TABLE[i] = Dot.to_dotionary(json[i])

TIME_TO_LIVE:int = 60 * 30
'''30 minutes cache lifetime'''

FACILITY_CACHE:list = [
    {
        "date":None,        ## ISO date format
        "fetch_time":None,  ## seconds since epoch, int
        "latency":None,     ## time taken to fetch table
        "dataframe":None    ## Date table
    } for i in range(len(FACILITY_TABLE))
]
'''Cache vector for src facilities.
For nerds, this implementation is similar to a directly-mapped cache.
Will change to 2-way set associative if demand is high.'''


def parse_date(date_str:str) -> date:
    '''Parse input string into a date object.\n
    Invalid dates are interpreted as the current day'''
    try:
        date_obj = parse(date_str)
    except:
        date_obj = date.today()

    return date_obj


def strtoint(data):
    '''Attempts to convert a string data type into int'''
    try:
        return int(data)
    except:
        return data


def show_facility_table() -> str:
    '''Returns a dataframe that contains facilities to be viewed'''

    returndf = pd.DataFrame({"no":[], "facility":[]})
    for i in range(len(FACILITY_TABLE)):
        # returndf = returndf.append(pd.DataFrame({"no":[i+1], "facility":[config[i].name]}))
        returndf = pd.concat([returndf, pd.DataFrame({"no":[i+1], "facility":[FACILITY_TABLE[i].name]})])

    returndf = returndf.convert_dtypes() ## set to ints
    return returndf.to_string(index=False)


def return_facility_list_shortform()->list:
    returnlist = [FACILITY_TABLE[i].shortname for i in range(len(FACILITY_TABLE))]
    # print(FACILITY_TABLE[0].shortname)
    return returnlist


def get_booking_table(date:date, tablecol:int) -> pd.DataFrame:
    '''Returns the booking table for a particular day, along with the next 7 days\n
    Raw table output, taken direct from page'''

    datereq = date.strftime('%d-%b-%y').upper()

    url = f"https://wis.ntu.edu.sg/pls/webexe88/srce_smain_s.srce$sel31_v?choice=1&fcode={FACILITY_TABLE[tablecol].codename}&fcourt={FACILITY_TABLE[tablecol].courts}&ftype=2&p_date={datereq}&p_mode=2"
    lg.functions.debug(f'url: {url}')

    table = pd.read_html(url)[0]
    return table


def format_booking_table(table:pd.DataFrame, facility_no:int, offset:int = 0)->pd.DataFrame:
    '''Formats the raw table into a summary table, with a specified offset.
    Offset is based on the start date of the booking table.'''

    ## blank dataframe
    avail_df = pd.DataFrame({'time':[],'slots':[]})
    table = table.iloc[:, [0,offset+2]]
    courts = FACILITY_TABLE[facility_no].courts

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

        avail_df = pd.concat([avail_df, pd.DataFrame({'time':[hour],'slots':[count]})])#, ignore_index=True)


    ## final formatting step
    for col in avail_df.columns: ## convert to respective data types
        avail_df.loc[:,col] = avail_df[col].apply(strtoint)
    avail_df = avail_df.astype(str) ## then convert all to strings
    avail_df.iloc[:,1] = avail_df.iloc[:,1].apply\
        (lambda x: x + f"/{courts}" if x.isnumeric() else x) ## additional, for ints

    return avail_df


def get_booking_result(date:date, facility_no:int) -> str:
    '''Main calling function.\n
    Returns formatted string to be printed/sent.\n
    Wraps the above 2 functions and formats the resulting string.'''
    ## fetch and format block
    t_start = time.time()
    table = get_booking_table(date, facility_no)
    t_end = time.time()
    table_f = format_booking_table(table, facility_no)

    exec_time = "{:5.4f}".format(t_end - t_start) ##in seconds, to check on fetch speed

    ## constructing string
    returnstr = f"\
{date.strftime('%d %b %y, %A')}\n\
{FACILITY_TABLE[facility_no].name}\n\n\
{table_f.to_string(index=False)}\n\n\
fetch time: {exec_time}s"

    return returnstr


def get_time_slots(tablecol:int)->pd.DataFrame:
    '''Return a DataFrame that corresponds to the time slots for a facility'''
    rawtable = get_booking_table(date.today(), tablecol)
    courts = FACILITY_TABLE[tablecol].courts
    indexes = [i*courts for i in range(int(len(rawtable)/courts))]
    table = rawtable.iloc[indexes, 0].reset_index(drop=True)

    returndf = pd.DataFrame({"no":[], "timeslot":[]})
    for i in range(len(table)):
        # returndf = returndf.append(pd.DataFrame({"no":[i+1], "timeslot":[table[i]]}))
        returndf = pd.concat([returndf, pd.DataFrame({"no":[i+1], "timeslot":[table[i]]})])

    return returndf.convert_dtypes().reset_index(drop=True)


def get_booking_result_cache(date_in:date, facility_no:int)->str:
    '''Functions the same as get_booking_result(), uses cached data if possible'''

    try:
        date_in_cache = date.fromisoformat(FACILITY_CACHE[facility_no]["date"])
    except:
        date_in_cache = None

    ## check for valid cache entry
    if date_in_cache is None:
        populate_cache(date_in, facility_no)

    elif date_in >= date_in_cache and\
        date_in <= date_in_cache + timedelta(days=7):

        if time.time() - FACILITY_CACHE[facility_no]["fetch_time"] < TIME_TO_LIVE:
            pass
        else:
            populate_cache(date_in, facility_no)

    else:
        populate_cache(date_in, facility_no)

    ## construct data
    booking_table:pd.DataFrame = copy.deepcopy(FACILITY_CACHE[facility_no]["dataframe"])
    date_in_cache = date.fromisoformat(FACILITY_CACHE[facility_no]["date"])
    offset = date_in - date_in_cache ## offset should be between 0 and 7

    lg.functions.debug(f"booking table offset by days: +{offset.days}")

    table_f:pd.DataFrame = format_booking_table(booking_table, facility_no, offset.days)
    exec_time = "{:5.4f}".format(FACILITY_CACHE[facility_no]["latency"])
    fetch_time = datetime.fromtimestamp(FACILITY_CACHE[facility_no]["fetch_time"])

    ## construct string
    returnstr = f"\
{date_in.strftime('%d %b %y, %A')}\n\
{FACILITY_TABLE[facility_no].name}\n\n\
{table_f.to_string(index=False)}\n\n\
last fetch time: {fetch_time.strftime('%H:%M')}\n\
time-to-fetch: {exec_time}s"

    return returnstr


def populate_cache(date_in:date, facility_no:int):
    '''Populates cache location with new data'''
    t_start = time.time()
    FACILITY_CACHE[facility_no]["dataframe"] = get_booking_table(date_in, facility_no)
    t_end = time.time()
    FACILITY_CACHE[facility_no]["fetch_time"] = t_end
    FACILITY_CACHE[facility_no]["latency"] = t_end - t_start
    FACILITY_CACHE[facility_no]["date"] = date_in.isoformat()
    return


def update_existing_cache_entries_sync():
    '''Updates any existing entries in the cache vector.'''

    for index in range(len(FACILITY_CACHE)):
        if FACILITY_CACHE[index]["date"] is not None:
            populate_cache(
                date.fromisoformat(FACILITY_CACHE[index]["date"]),
                index
            )

    return
