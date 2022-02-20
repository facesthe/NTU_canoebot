import pandas as pd
import numpy as np
from datetime import date, timedelta
from dateutil.parser import parse
from dateutil.relativedelta import relativedelta
import os
from lib.liblog import loggers as lg ## new logging extension
import settings as s ## bot settings

lg.functions.debug("sheetscraper loaded")

global SHEET_ID

## Update this if needed, or sheetscraper won't work!
## change the settings in botsettings.json
SHEET_ID =              s.json.sheetscraper.attendance_sheet            ## current sheet id for AY21/22
SHEET_PROG =            s.json.sheetscraper.program_sheet               ## training prog sheet AY21/22
DECONFLICT =            s.json.sheetscraper.use_deconflict              ## boat deconflict enable/disable
DECONFLICT_VERSION =    s.json.sheetscraper.deconflict_ver              ## version 1 or 2 of __deconflict
RECURSION_LIMIT =       s.json.sheetscraper.deconflict_recursion_limit  ## set the recursion limit for deconflict()


## create data, attendance folders on first run
if not os.path.exists('./data'):
    os.mkdir('./data')
    lg.functions.info('creating /data')
lg.functions.info('/data already created')
if not os.path.exists('./data/attendance'):
    lg.functions.info('creating /data/attendance')
    os.mkdir('./data/attendance')
lg.functions.info('/data/attendance already created')

## Functions are defined from least dependent to most

## basic function for importing google sheet data as a dataframe
## First row taken to be col headers for dataframe
## sheet name is optional, not needed if document contains only one sheet
## if document contains multiple sheets, first sheet created will be passed
def getgooglesheet(sheet_id, sheet_name=''):
    url = f'https://docs.google.com/spreadsheets/d/{sheet_id}/gviz/tq?tqx=out:csv&sheet={sheet_name}'
    df_raw = pd.read_csv(url, header=None)
    df_raw = df_raw.rename(columns=df_raw.iloc[0])
    df_raw = df_raw.drop(0)
    df_raw = df_raw.reset_index(drop=True)

    return df_raw


## For training program sheet (separate from paddling attendance)
def gettrainingprog(date_in:date):
    prog_table = getgooglesheet(SHEET_PROG)
    prog_day = prog_table[prog_table['Date'].str.match(date_in.strftime('%Y-%m-%d'))]
    prog_day = prog_day.iloc[:,[2,3]].reset_index(drop=True)
    prog_day = prog_day.convert_dtypes()

    return prog_day


## top level function used by canoebot
## wraps gettrainingprog with input validation
## am version
def trainingam(str_in):
    try:
        date_in = parse(str_in)
    except:
        date_in = date.today()

    prog = gettrainingprog(date_in)['AM program']
    if prog.isna()[0]:
        prog = pd.Series(['none'])

    prog = stackontop(prog, date_in.strftime('%a %d %b'))
    return df2str(prog)


## top level function used by canoebot
## wraps gettrainingprog with input validation
## pm version
def trainingpm(str_in):
    try:
        date_in = parse(str_in)
    except:
        date_in = date.today()

    prog = gettrainingprog(date_in)['PM program']
    if prog.isna()[0]:
        prog = pd.Series(['none'])

    prog = stackontop(prog, date_in.strftime('%a %d %b'))
    return df2str(prog)


## calculate the last sunday that is still within the month
def getlastsun(date_in:date)->date: ## date_in is a date object
    '''Calculate last sunday that is still withihn the month.
    Input: date in in datetime.date format'''
    month_max = date(date_in.year,date_in.month,1) + relativedelta(months=+1) - timedelta(days=1)

    if month_max.isoweekday() != 7:
        last_sun = month_max + timedelta(days=-month_max.isoweekday())
    else:
        last_sun = month_max

    return last_sun


def getfirstmon(date_in:date)->date:
    '''Calculate the first monday in a sheet month.\n
    Note that this monday may be from the previous month.
    If the first day of the month is not a monday, use the previous weeks monday'''

    ## first day of month
    monthstart = date(date_in.year, date_in.month, 1)
    if monthstart.isoweekday != 1:
        first_mon = monthstart + timedelta(days = -monthstart.isoweekday()+1)
    else:
        first_mon = monthstart

    return first_mon


def getsheetdate(date_in:date)->date:
    '''Get the sheet header date for a given date'''
    sheet_end = getlastsun(date_in)

    if (date_in - sheet_end).days > 0: ## exceeded this month, use next month
        return sheet_end + timedelta(days=1)
    else:
        return getfirstmon(date_in)


## get the sheet name for a particular date, in string form
def getsheetname(date_in:date): ## date_in is a date object again
    if date_in > getlastsun(date_in):
        sheet_name = (date_in+timedelta(days=7)).strftime('%b-%Y')
    else:
        sheet_name = date_in.strftime('%b-%Y')

    lg.functions.debug(f'{date_in} resolved to sheetname {sheet_name}')
    return sheet_name


## returns dataframe for entire month, no modifications
def getsheet(date_in:date):
    '''Returns the chosen month as a dataframe.

    Note that the start and end of the month are not consistent.'''
    url = f'https://docs.google.com/spreadsheets/d/{SHEET_ID}/gviz/tq?tqx=out:csv&sheet={getsheetname(date_in)}'
    df_raw = pd.read_csv(url, header=None)
    df_raw.drop(columns=[0],inplace=True)
    df_raw.columns = range(df_raw.shape[1])
    # print(df_raw.head())
    #df_raw = df_raw.style.set_properties(**{'text-align': 'left'})

    return df_raw


def getconfigsheet():
    url = f'https://docs.google.com/spreadsheets/d/{SHEET_ID}/gviz/tq?tqx=out:csv&sheet=configs'
    return pd.read_csv(url).iloc[:,1:]


## creates the dictionary to be used by the name shortener/lengthener
def createnamesdict():
    global SHORT_NAME
    global LONG_NAME
    df = pd.read_csv('./data/names.csv')
    df = df.set_index('name')['shortname']
    SHORT_NAME = df.dropna().to_dict()
    LONG_NAME = {value:key for (key,value) in SHORT_NAME.items()}


## Fetch the latest boat config and namelist from Gsheets
## shorten names (if applicable) in the boat df, then export
def updateconfigs():
    lg.functions.debug('updating boats.csv and names.csv in ./data')
    df_raw = getconfigsheet()
    df_raw.iloc[:,:2].to_csv('./data/names.csv',encoding='utf-8',index=False)

    createnamesdict()

    df_raw = df_raw.iloc[:,[0,2,3]]
    for i in range(len(df_raw['name'])):
        if df_raw['name'][i] in SHORT_NAME:
            df_raw['name'][i] = SHORT_NAME[df_raw['name'][i]]
    df_raw.to_csv('./data/boats.csv',encoding='utf-8',index=False)


def create_1star_dict():
    updateconfigs() ## get the latest information
    global CERT_STATUS
    df = getconfigsheet()[['name','1_star']]

    for i in range(len(df['name'])):
        if df.loc[i, 'name'] in SHORT_NAME:
            df.loc[i, 'name'] = SHORT_NAME[df.loc[i, 'name']]

    df = df.set_index('name')['1_star']
    CERT_STATUS =  df.dropna().to_dict()


## Returns a formatted dataframe containing names, the date and session
## Names that can be shortened will be shortened
## Depreciated
def getnames(str_in,time:int):
    global SHORT_NAME
    ## time passed as int, am=0,pm=1
    ## no time param passed, assume 0

    ## no date passed, default to next day
    try:
        date_in = parse(str_in).date()
    except:
        date_in = date.today() + timedelta(days=1)

    if len(str_in) == 0: str_in = 'empty string'
    lg.functions.debug(f"{str_in} interpreted as {date_in}")

    raw_sheet = getsheet(date_in)
    lg.functions.debug('first cell:',raw_sheet.iloc[0,0])
    ## no longer require the count for males and females
    vertoffset = findlowestname(raw_sheet) + 2 ## dynamic, changes based on how many names are in excel

    sheet_start_date = parse(raw_sheet.iloc[1,0]) ## make sure that sheet start date is input correctly in Gsheets
    raw_sheet = raw_sheet.iloc[:,1:]
    lg.functions.debug(sheet_start_date) ## for debugging
    ## get the relative location of the col corresponding to date
    delta = (date_in - sheet_start_date.date()).days ## in days
    wekindex = delta // 7
    dayindex = delta % 7
    amoffset = 17*wekindex + 2*dayindex + 3
    #pmoffset = amoffset + 1 ## no need to assign another var

    if(not time):
        timestr = 'AM'
        df_session = raw_sheet.iloc[vertoffset:,amoffset]
    else:
        timestr = 'PM'
        df_session = raw_sheet.iloc[vertoffset:,amoffset + 1]

    ## insert session details at top of dataframe
    df_session = df_session.dropna().reset_index(drop=True)

    for i in range(len(df_session)):
        if df_session[i] in SHORT_NAME:
            df_session[i] = SHORT_NAME[df_session[i]]
    df_session = stackontop(df_session, '')
    df_session = stackontop(df_session, date_in.strftime(f'%a {timestr}'))
    df_session = stackontop(df_session, date_in.strftime(f'%d %b %y'))

    return df_session
    ## return dataframe containing names

## Obtain the full names for a paddling session
## No date and time information included in the Series
def getonlynames(str_in, time:int):

    try:
        date_in = parse(str_in).date()
    except:
        date_in = date.today() + timedelta(days=1)

    if len(str_in) == 0: str_in = 'empty string'
    lg.functions.debug(f"{str_in} interpreted as {date_in}")

    ## new code part from here onwards
    raw_sheet = getsheet(date_in)
    sheet_start_date = getsheetdate(date_in)
    lg.functions.debug(f'sheet start date: {sheet_start_date}')
    # sheet_start_date = parse(raw_sheet.iloc[1,0]).date()

    ## get the relative location of the col corresponding to date
    delta = (date_in - sheet_start_date).days ## in days
    wekindex = delta // 7
    dayindex = delta % 7
    offset = 17*wekindex + 2*dayindex + 4

    if time:
        offset += 1

    df_session = pd.Series()
    lg.functions.debug(f'looping in range of 3 to {len(raw_sheet)}')
    lg.functions.debug(f'offset col: {offset}')

    ## build df_session
    names = [] ## temp list
    #lg.functions.debug(raw_sheet.iloc[:, offset])

    for row in range(3, len(raw_sheet)):
        if(str(raw_sheet.iloc[row, offset]).upper() == 'Y'):
            names.append(raw_sheet.iloc[row, 0])

    ## add names if there are any
    if len(names) != 0:
        df_session = pd.concat([df_session, pd.Series(names)])

    lg.functions.debug(f'names: {names}')
    return df_session.reset_index(drop=True)


def getnamesv2(str_in, time:int):
    global SHORT_NAME

    try:
        date_in = parse(str_in).date()
    except:
        date_in = date.today() + timedelta(days=1)

    if len(str_in) == 0: str_in = 'empty string'
    lg.functions.debug(f"{str_in} interpreted as {date_in}")

    ## new code part from here onwards
    raw_sheet = getsheet(date_in)
    sheet_start_date = getsheetdate(date_in)
    lg.functions.debug(f'sheet start date: {sheet_start_date}')
    # sheet_start_date = parse(raw_sheet.iloc[1,0]).date()

    ## get the relative location of the col corresponding to date
    delta = (date_in - sheet_start_date).days ## in days
    wekindex = delta // 7
    dayindex = delta % 7
    offset = 17*wekindex + 2*dayindex + 4

    if(not time):
        timestr = 'AM'
    else:
        timestr = 'PM'
        offset += 1

    df_session = pd.Series([
        date_in.strftime(f'%d %b %y'), \
        date_in.strftime(f'%a {timestr}'), \
        ''
        ])
    lg.functions.debug(f'looping in range of 3 to {len(raw_sheet)}')
    lg.functions.debug(f'offset col: {offset}')

    ## build df_session
    names = [] ## temp list
    #lg.functions.debug(raw_sheet.iloc[:, offset])

    for row in range(3, len(raw_sheet)):
        if(str(raw_sheet.iloc[row, offset]).upper() == 'Y'):
            names.append(raw_sheet.iloc[row, 0])

    ## shorten and add names (if any)
    if len(names) != 0:
        for i in range(len(names)):
            if names[i] in SHORT_NAME:
                names[i] = SHORT_NAME[names[i]]

        df_session = pd.concat([df_session, pd.Series(names)])
    lg.functions.debug(f'names: {names}')
    return df_session.reset_index(drop=True)


def namelist(date_time_str=''):

    date_time = date_time_str.split(',')
    #print(datetime)

    ## check if there are 2 elements in list
    try:
        date_time[1]
    except:
        if s.json.sheetscraper.getnames_ver == 2:
            return getnamesv2(date_time[0], 0)
        else:
            return getnames(date_time[0], 0)

    ## if 2 elements check if it says 'pm'
    if date_time[1].strip().lower() in ['pm','aft','afternoon']:
        lg.functions.debug("using names for PM slot")
        if s.json.sheetscraper.getnames_ver == 2:
            return getnamesv2(str_in=date_time[0],time=1)
        else:
            return getnames(str_in=date_time[0],time=1)
    else:
        lg.functions.debug("using names for AM slot")
        if s.json.sheetscraper.getnames_ver == 2:
           return getnamesv2(date_time[0], 0)
        else:
            return getnames(date_time[0], 0)


## pair up the boats with names
def match2boats(df_session):

    boatlist = ['' for i in range(3)] ## 3 top rows used for description
    boats = pd.read_csv('./data/boats.csv')
    session_list = df_session.tolist()

    for i in range(len(boats)):
        if boats['name'][i] in session_list:
            boatlist.append(boats['boat_1'][i])

    ## construct new dataframe
    return_df = pd.DataFrame({
        'col1':session_list,
        'col2':boatlist
    })

    ## perform recursive deconflict here (recursive part not fully implemented)
    if DECONFLICT: ## enable by changing this to TRUE (top of file)
        predeconflict() ## reset the recursion counter
        return_df = deconflict(return_df)

    ## adjust date headers to be side-by-side
    return_df.iloc[0,1]=return_df.iloc[1,0]
    return_df.iloc[1,0]=''
    return_df.drop(1,inplace=True)
    return_df.columns = return_df.iloc[0]
    return return_df[1:].reset_index(drop=True)


def boatallo(str_in=''):
    '''Boat allo function that is used by the canoebot\n
    No params -> next day used\n
    Input [date, time] as string\n
    [time] is 'pm' - ommit [time] if am session\n
    See other functions called in here for more details'''
    tempdf = namelist(str_in)
    return match2boats(tempdf)


## recursive deconflict
## takes in unprocessed boat allo dataframe
## check for any matching boats, change the boat allo so that all are unique (if possible)
def deconflict(df_in):
    ## take the 2nd col of dataframe
    ## ignore the top 3 blank entries
    ## generate a frequency df
    global COUNT
    freq = df_in.iloc[:,1].value_counts().iloc[1:]

    ## no conflicting boats, exit
    ## accounts for unresolvable conflicts
    count = 0
    for i in range(len(freq)):
        if freq[i] == 1:
            count += 1
        elif 'CONFLICT' in freq.reset_index().iloc[i,0]:
            count += 1

    if count == len(freq):
        return df_in ## return the same dataframe that was given, no operations performed
    elif COUNT != 0: ## recursion limit of COUNT
        COUNT -= 1
    else:
        lg.functions.info(f'recursion limit of {RECURSION_LIMIT} reached')
        return __automarkconflict(df_in)

    ## has conflicting boats
    ## build list of conflicting boats
    conflicts = []
    freqlist = freq.reset_index()
    for i in range(len(freq)):
        if freq[i] > 1 and 'CONFLICT' not in freqlist.iloc[i,0]: ## filter out conflict
            conflicts.append(freqlist.iloc[i,0])
        else: continue

    ## perform the actual deconflict (__deconflict is not complete)
    for boat in conflicts:
        if DECONFLICT_VERSION == 1:
            lg.functions.info('using deconflict version 1')
            df_in = __deconflict(df_in, boat)
        elif DECONFLICT_VERSION == 2:
            lg.functions.info('using deconflict version 2')
            df_in = __deconflictv2(df_in, boat)

    return deconflict(df_in)


## helper function
## does the search and replacement of boats
def __deconflict(df_in, conflictboat):
    '''search and find a replacement for problem boat'''

    boatlist = pd.read_csv('./data/boats.csv') ## retrieve the boat allo

    ## build list of conflict names for particular boat
    conflictnames = []
    for i in range(len(boatlist)):
        if conflictboat == boatlist.boat_1[i]:
            conflictnames.append(boatlist.name[i])
    lg.functions.debug(f"conflicting boat: {conflictboat}")
    lg.functions.debug(f"conflictning names: {conflictnames}")

    for i in range(len(conflictnames)):
        row = boatlist.loc[boatlist.name == conflictnames[i]] ## get the row of boat_1 and boat_2

        if pd.isnull(row.iloc[0,2]): ## check if boat_2 has a boat
            continue
        elif row.iloc[0,0] not in df_in.col1.tolist(): ## name not in boat allo
            lg.functions.debug(f"{row.iloc[0,0]} not in boat allo, removing")
            continue
        else: ## perform replacement here
            ## has to be done in 2 steps cause python won't let me
            replacement = boatlist.loc[boatlist.name == conflictnames[i]].iloc[0,2] ## boat_2
            if df_in.loc[df_in.col1 == conflictnames[i]].iloc[0,1] == replacement:
                continue
            else:
                df_in.loc[df_in.col1 == conflictnames[i],'col2'] = replacement
            lg.functions.info('deconflict successful')
            return df_in ## successful replacement

    lg.functions.info('deconflict failed')
    return __markconflict(df_in, conflictboat) ## all attempts failed, mark the boats


## helper function
## improved version of __deconflict
def __deconflictv2(df_in, conflictboat):
    '''improved version of __deconflict\n
    Right now this can go on an infinite loop'''

    boatdf = pd.read_csv('./data/boats.csv') ## retrieve the boat allo

    ## build list of conflict names for particular boat
    ## use set logic to build list
    conflictnames = []
    conflictboatset = set() ## set of boats to consider deconflicting
    #conflictnames = df_in.col1[3:].tolist()

    for i in range(len(boatdf)):
        ## add name if matching boat
        ## update the conflict boat set
        if conflictboat in boatdf.iloc[i, 1:].tolist():
            conflictnames.append(boatdf.iloc[i,0])
            conflictboatset.update(tuple(boatdf.iloc[i, 1:].tolist()))
            conflictboatset = set(filter(lambda x: x == x, conflictboatset))
        ## add name if any boat matches the set
        elif boatdf.iloc[i,1] in conflictboatset or boatdf.iloc[i,2] in conflictboatset:
            conflictnames.append(boatdf.iloc[i,0])
            conflictboatset.update(tuple(boatdf.iloc[i, 1:].tolist()))
            conflictboatset = set(filter(lambda x: x == x, conflictboatset))

    lg.functions.debug(f"all possible conflicting boats: {conflictboatset}")

    ## remove names not in current boat allo
    for i in range(len(conflictnames)-1, -1, -1):
        #print(f"checking {conflictnames[i]}...")
        if conflictnames[i] not in df_in.iloc[3:,0].tolist():
            #print(f"removing {conflictnames[i]} from {conflictnames}\nas it doesnt exist in {df_in.iloc[3:,0].tolist()}")
            conflictnames.remove(conflictnames[i])

    # for i in range(3,len(df_in)):
    #     line = boatdf.loc[boatdf.name == df_in.iloc[i,0]]
    #     #print(line)
    #     if df_in.iloc[i,1] == conflictboat: ## if boat in boatallo matches the conflict
    #         conflictnames.append(df_in.iloc[i,0])
    #     elif conflictboat in line.iloc[0,1:].tolist(): ## if person has boat 1 or 2 as conflictboat
    #         conflictnames.append(df_in.iloc[i,0])

    ## build an array of boats 1 and 2 for each person
    boatarray = []
    for i in range(len(conflictnames)):
        boatarray.append(boatdf.loc[boatdf.name == conflictnames[i]].iloc[0,1:].tolist())

    lg.functions.debug(f"conficting boat: {conflictboat}")
    lg.functions.debug(f'conflicting names: {conflictnames}')
    lg.functions.debug(f'all boat combinations: {boatarray}')

    ## try all binary conbinations of boats using black magic
    for x in range(2 ** len(conflictnames)):
        testlist = [] ## empty list for testing
        testbinary = format(x, f'0{len(conflictnames)}b') ## testbinary is formatted as string
        ## build the test list
        for i in range(len(conflictnames)):
            testlist.append(boatarray[i][int(testbinary[i])])
        lg.functions.debug(f'attempting combination: {testlist}')

        ## if all occurences are unique, use that
        if __islistunique(testlist):
            lg.functions.info('deconflict successful')
            lg.functions.debug(f"new boat assignment:\n{conflictnames}\n{testlist}")
            return __replaceboat(df_in, conflictnames, testlist)

    ## unsuccessful deconflict, mark boats
    lg.functions.info('deconflict failed')
    return __markconflict(df_in, conflictboat)


def predeconflict():
    global COUNT
    COUNT = RECURSION_LIMIT ## set recursion counter


def __replaceboat(df_in, names_in, boats_in) -> pd.DataFrame:
    '''override boats that match the index of the name list passed.\n
    Size of name and boat list must match'''
    for i in range(len(names_in)):
        df_in.loc[df_in.col1 == names_in[i],'col2'] = boats_in[i]

    return df_in


def __islistunique(list_in) -> int:
    '''Return 1 if list contains unique entries. Return 0 otherwise.'''
    if np.nan in list_in:
        return 0
    elif len(set(list_in)) == len(list_in):
        return 1
    else: return 0


## helper function
## appends 'CONFLICT' to boats that are unable to be deconflicted
def __markconflict(df_in, conflictboat) -> pd.DataFrame:
    for i in range(len(df_in)):
        if df_in.iloc[i,1] == conflictboat:
            df_in.iloc[i,1] += ' CONFLICT'
    return df_in


def __automarkconflict(df_in) -> pd.DataFrame:
    freq = df_in.iloc[3:,1].value_counts()
    conflicts = []
    freqlist = freq.reset_index()

    for i in range(len(freq)):
        if freq[i] > 1 and 'CONFLICT' not in freqlist.iloc[i,0]:
            conflicts.append(freqlist.iloc[i,0])
        else: continue

    lg.functions.debug(f"automark conflict detected unresolvable conflict(s): {conflicts}")

    for boat in conflicts:
        __markconflict(df_in, boat)

    return df_in


###### Dataframe/string manipulation ######
## adds some backticks to turn text into code form
def codeit(str_in) -> str:

    return '```\n' + str_in + '\n```'


## formats df into string nice and tidy, for telegram bot to send
##
def df2str(df_in) -> str:

    return codeit(string_df(df_in))


## convert dataframe or series into string form
## justify all left
def string_df(df_in) -> str:

    if type(df_in) == pd.Series:
        df_in = pd.Series(df_in)
        maxlen = 0
        returnstr = ""

        ## first pass
        for entry in df_in:
            maxlen = max(maxlen, len(entry))

        ## second pass
        for entry in df_in:
            entry = str(entry)
            entry.ljust(maxlen)
            returnstr += entry + "\n"

        return returnstr

    elif type(df_in) == pd.DataFrame:
        df_in = pd.DataFrame(df_in)
        maxlen = []
        index = 0
        returnstr = ""

        ## first pass
        for col in list(df_in.columns):
            maxlen.append(len(col))
            for item in df_in[col]:
                maxlen[index] = max(maxlen[index], len(str(item)))
            index += 1

        ## second pass
        collist = list(df_in.columns)
        for k in range(len(collist)):
            returnstr += str(collist[k]).ljust(maxlen[k]) + "  "

        returnstr += "\n"
        cols = df_in.shape[1]

        ## third pass
        for i in range(len(df_in)):
            row = ''
            for j in range(cols):

                row += str(df_in.iloc[i, j]).ljust(maxlen[j]) + "  "
            returnstr += row + "\n"

        return returnstr

    else:
        return None


## for adding entries on top of a pandas series
## index reset to fit
def stackontop(df_in,item):
    df_in.loc[-1] = item
    df_in.index += 1
    return df_in.sort_index()


## looks for the lowest name in the sheet, returns the row index for that name
## use this function on getsheet()
def findlowestname(df_in) -> int:
    index = 0
    while(True):
        if pd.isnull(df_in.iloc[index+1,0]) and pd.isnull(df_in.iloc[index+2,0]):
            return index
        else:
            index += 1
            continue

## run update on import/load
updateconfigs()
