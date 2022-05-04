import json as jsn
import pandas as pd
import time
import threading
from datetime import datetime, date, timedelta
from dateutil.parser import parse
import copy

import lib.liblog as lg
import lib.Dotionary as Dot

lg.functions.debug("srcscraper loaded")

_path = './.configs/srcscraper.config.json' ## path to srcscraper.config

SRC_LINK = 'https://sso.wis.ntu.edu.sg/webexe88/owa/sso_login1.asp?t=1&p2=https://wis.ntu.edu.sg/pls/webexe88/srce_smain_s.Notice_O&extra=&pg='

## creating config variable in Dotionary form
with open(_path) as jsonfile:
    json = jsn.load(jsonfile)

FACILITY_TABLE = [Dot for i in range(len(json))]
'''Contains SRC facility info in an array of dot-notation accessible dictionaries.'''

for i in range(len(FACILITY_TABLE)):
    FACILITY_TABLE[i] = Dot.to_dotionary(json[i])

TIME_TO_LIVE_LONG:int = 60 * 60
'''60 minutes cache lifetime'''

TIME_TO_LIVE_SHORT:int = 60 * 5
'''cache refresh interval when entering main menu'''

FACILITY_CACHE:list = [
    [
        {
            "date":         None, ## date object
            "fetch_time":   None, ## seconds since epoch, int
            "latency":      None, ## time taken to fetch table, seconds
            "old":          None, ## age of cache line, relative [T/F]
            "dataframe":    None  ## data table
        } for x in range(2)
    ] for i in range(len(FACILITY_TABLE))
]
'''Cache vector for src facilities.
For nerds, this implements a modified 2-way set associative cache.'''

SRC_CACHE_MUTEX = threading.Lock()
'''Cache mutex. Don't want to have corrupted data'''


def parse_date(date_str:str) -> date:
    '''Parse input string into a date object.\n
    Invalid dates are interpreted as the current day'''
    try:
        date_obj = parse(date_str)
    except:
        date_obj = date.today()

    return date_obj


def touppercase(data:any):
    try:
        data:str = str(data)
        return data.upper()
    except:
        return data


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


def get_booking_table(date_in:date, facility_no:int) -> pd.DataFrame:
    '''Returns the booking table for a particular day, along with the next 7 days\n
    Raw table output, taken direct from page'''

    datereq = date_in.strftime('%d-%b-%y').upper()

    url = f"https://wis.ntu.edu.sg/pls/webexe88/srce_smain_s.srce$sel31_v?choice=1&fcode={FACILITY_TABLE[facility_no].codename}&fcourt={FACILITY_TABLE[facility_no].courts}&ftype=2&p_date={datereq}&p_mode=2"
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

        ## convert all entries to uppercase strings
        ## because SRC has both 'closed' and 'CLOSED' for some reason.
        for col in data_hr:
            data_hr.loc[:,col] = data_hr[col].apply(touppercase)

        try:
            count = data_hr.value_counts()['AVAIL']
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
    '''Main calling function. DEPRECEATED, use get_booking_result_cache() instead.\n
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


def get_time_slots(facility_no:int)->pd.DataFrame:
    '''Return a DataFrame that corresponds to the time slots for a facility'''
    rawtable = get_booking_table(date.today(), facility_no)
    courts = FACILITY_TABLE[facility_no].courts
    indexes = [i*courts for i in range(int(len(rawtable)/courts))]
    table = rawtable.iloc[indexes, 0].reset_index(drop=True)

    returndf = pd.DataFrame({"no":[], "timeslot":[]})
    for i in range(len(table)):
        # returndf = returndf.append(pd.DataFrame({"no":[i+1], "timeslot":[table[i]]}))
        returndf = pd.concat([returndf, pd.DataFrame({"no":[i+1], "timeslot":[table[i]]})])

    return returndf.convert_dtypes().reset_index(drop=True)


def get_booking_result_cache(date_in:date, facility_no:int)->str:
    '''Functions the same as get_booking_result(), but with caching'''

    ## construct data
    cache_entry:pd.DataFrame = copy.deepcopy(get_table_from_cache(date_in, facility_no))
    date_in_cache:date = cache_entry["date"]
    offset = (date_in - date_in_cache).days ## offset should be between 0 and 7

    lg.functions.debug(f"booking table offset by days: +{offset}")

    table_f:pd.DataFrame = format_booking_table(cache_entry["dataframe"], facility_no, offset)
    exec_time = "{:5.4f}".format(cache_entry["latency"])
    fetch_time = datetime.fromtimestamp(cache_entry["fetch_time"]).strftime('%H:%M')

    ## construct string
    returnstr = f"{date_in.strftime('%d %b %y, %A')}\n"\
    f"{FACILITY_TABLE[facility_no].name}\n\n"\
    f"{table_f.to_string(index=False)}\n\n"\
    f"last fetch time: {fetch_time}\n"\
    f"time-to-fetch: {exec_time}s\n"

    return returnstr


def get_cache_stored_dates(facility_no:int):
    '''Returns a list of dates for a particular facility that are populated in cache'''
    cache_lines = FACILITY_CACHE[facility_no]
    returnlist:list[date] = []

    for i in range(len(cache_lines)):
        if cache_lines[i]["date"] is None:
            lg.functions.debug(f"cache line {facility_no}:{i} has no entry")
            continue
        else:
            lg.functions.debug(f"cache line {facility_no}:{i} has entry")
            for item in [
                (cache_lines[i]["date"] + timedelta(days=x))
                for x in range(8) ## every start day has 7 more days ahead of it
            ]:
                returnlist.append(item)

    return returnlist


def get_cache_line_no(date_in:date, facility_no:int)->int:
    '''Get the cache line that holds a specified date. Returns -1 if unsuccessful'''
    for i in range(len(FACILITY_CACHE[facility_no])):
        cache_date:date = FACILITY_CACHE[facility_no][i]["date"]
        if 0<= (date_in - cache_date).days <= 7:
            return i
    return -1


def fill_cache(date_in:date, facility_no:int, cache_line:int):
    '''Populates specified cache location with data. Flips the other cache line age to true'''
    SRC_CACHE_MUTEX.acquire()
    FACILITY_CACHE[facility_no][cache_line]["date"] = date_in
    FACILITY_CACHE[facility_no][cache_line]["fetch_time"] = 0   ## set to the beginning of time
    FACILITY_CACHE[facility_no][cache_line]["old"] = False      ## set itself to new
    if FACILITY_CACHE[facility_no][1-cache_line]["date"] is not None:
        FACILITY_CACHE[facility_no][1-cache_line]["old"] = True ## set other to old
    SRC_CACHE_MUTEX.release()
    update_single_cache_entry(facility_no, cache_line)
    return


def fill_all_cache_sets_threaded():
    '''Fills one cache line in every set, taking the current day as reference.'''

    current_day = date.today()
    thread_vector:list[threading.Thread] = [
        threading.Thread(
            target=fill_cache,
            args=(
                current_day,
                facility_no,
                0
            )
        )
        for facility_no in range(len(FACILITY_TABLE))
    ]
    t_start = time.time()
    for subthread in thread_vector:
        subthread.start()
    for subthread in thread_vector:
        subthread.join()
    t_end = time.time()

    lg.functions.debug(f"Fill time for {len(thread_vector)} threads: ""{:5.4f}s".format(t_end - t_start))
    return


def get_table_from_cache(date_in:date, facility_no:int)->dict:
    '''Returns a matching cache entry for a given facility number and date.
    Performs cache replacement if necessary.'''

    stored_dates = get_cache_stored_dates(facility_no)

    if len(stored_dates) == 0: ## empty cache
        fill_cache(date_in, facility_no, cache_line=0)
        cache_line=0

    elif date_in in stored_dates: ## cache entry found
        cache_line = get_cache_line_no(date_in, facility_no)
        ## check if outdated, re-fetch if needed
        if (time.time() - FACILITY_CACHE[facility_no][cache_line]["fetch_time"]) > TIME_TO_LIVE_LONG:
            fill_cache(
                FACILITY_CACHE[facility_no][cache_line]["date"],
                facility_no,
                cache_line
            )

    elif date_in not in stored_dates: ## no cache entry found

        for i in range(len(FACILITY_CACHE[facility_no])):

            ## if there is empty/old slot (mutually exclusive), fill
            if FACILITY_CACHE[facility_no][i]["old"] is None\
                or FACILITY_CACHE[facility_no][i]["old"] == True:

                ## determine date to fetch
                cache_date_max = max(stored_dates)
                cache_date_min = min(stored_dates)

                lg.functions.debug(f'overwriting/populating cache line {facility_no}:{i}')

                if 1 <= (date_in - cache_date_max).days <= 8:
                    fill_cache(cache_date_max+timedelta(days=1), facility_no, i)
                elif 1 <= (cache_date_min - date_in).days <= 8:
                    fill_cache(cache_date_min+timedelta(days=-8), facility_no, i)
                else:
                    fill_cache(date_in, facility_no, i)

                cache_line = i
                break

    FACILITY_CACHE[facility_no][cache_line]["old"] = False
    if FACILITY_CACHE[facility_no][1-cache_line]["date"] is not None:
        FACILITY_CACHE[facility_no][1-cache_line]["old"] = True     ## set other to old

    return FACILITY_CACHE[facility_no][cache_line]


def update_single_cache_entry(facility_no:int, cache_line:int):#->bool:
    '''Worker function for threaded update. Given cache location must have an existing entry.
    Refreshes data in cache, no modifications to age or date made.'''
    # Returns True if a cache update has been made'''
    global SRC_CACHE_MUTEX

    SRC_CACHE_MUTEX.acquire()
    cache_line_data = FACILITY_CACHE[facility_no][cache_line]
    target_date = cache_line_data["date"]
    t_start = time.time()

    if t_start - FACILITY_CACHE[facility_no][cache_line]["fetch_time"] <= TIME_TO_LIVE_SHORT:
        lg.functions.debug(f'cache line {facility_no}:{cache_line} not updated: within {TIME_TO_LIVE_SHORT}s')
        SRC_CACHE_MUTEX.release()
        return ## False
    SRC_CACHE_MUTEX.release()

    data_table = get_booking_table(target_date, facility_no)
    t_end = time.time()

    SRC_CACHE_MUTEX.acquire()
    FACILITY_CACHE[facility_no][cache_line]["dataframe"] = data_table
    FACILITY_CACHE[facility_no][cache_line]["fetch_time"] = t_end
    FACILITY_CACHE[facility_no][cache_line]["latency"] = t_end - t_start
    SRC_CACHE_MUTEX.release()

    lg.functions.debug(f'cache line {facility_no}:{cache_line} updated: {t_end - t_start}')

    return ## True


def update_existing_cache_entries_threaded():
    '''Multithreaded update to all existing cache entries.'''
    thread_vector:list[threading.Thread] = []

    ## create thread array
    for facility_no in range(len(FACILITY_CACHE)):
        for cache_line in range(len(FACILITY_CACHE[facility_no])):
            if FACILITY_CACHE[facility_no][cache_line]["date"] is not None:
                thread_vector.append(
                    threading.Thread(
                        target=update_single_cache_entry,
                        args=(facility_no, cache_line)
                    )
                )

    t_start = time.time()

    ## start threads
    for subthread in thread_vector:
        subthread.start()
    ## join threads
    for subthread in thread_vector:
        subthread.join()

    t_end = time.time()
    lg.functions.debug(
        f"update time for {len(thread_vector)} thread(s): ""{:5.4f}s".format(t_end - t_start)
    )

    return
