import pandas as pd
import time
from datetime import date
from dateutil.parser import parse

from lib.liblog import loggers as lg

lg.functions.debug("gymscraper loaded")

HOUR_SLOTS = 20 ## can be changed for other bookings that have different slot size (STILL TESTING)

def gettable(date_in=''):
    ## possible arguments: date in any form imaginable
    try:
        dt = parse(date_in)
    except:
        ## no date specified, use current date
        dt = date.today()

    datereq = dt.strftime('%d-%b-%y').upper()
    src_url = f'https://wis.ntu.edu.sg/pls/webexe88/srce_smain_s.srce$sel31_v?choice=1&fcode=WG&fcourt={HOUR_SLOTS}&ftype=2&p_date={datereq}&p_mode='

    t_start = time.time()
    df_booking = pd.read_html(src_url)[0] ##takes the only table from src page
    t_end = time.time()
    exec_time = "{:5.4f}".format(t_end - t_start) ##in seconds, to check on fetch speed
    lg.functions.debug(f"fetch time of {exec_time}s")
    ## return the table from booking page
    return df_booking.iloc[:,[0,2]],dt, exec_time

def condense(df_in): ## takes the first date col and summarises the booking details as anther dataframe
    ## blank dataframe
    avail_df = pd.DataFrame({'time':[],'slots':[]})

    for multi in range(int(len(df_in)/HOUR_SLOTS)):
        ## hourly dataframe, each hour has 20 slots
        hour = df_in.iloc[multi*HOUR_SLOTS,0]
        data_hr = df_in.iloc[multi*HOUR_SLOTS:multi*HOUR_SLOTS+HOUR_SLOTS,[1]]

        try:
            count = data_hr.value_counts()['Avail']
        except:
            if (data_hr.value_counts()[0] == 20): ## if there ares 20 of the same entries
                count = data_hr.iloc[0,0]         ## take the name of that entry, e.g. NTU DBM
            else:
                count = 0

        avail_df = pd.concat([avail_df, pd.DataFrame({'time':[hour],'slots':[count]})], ignore_index=True)

    ## convert to int
    for col in avail_df.columns:
        avail_df.loc[:,col] = avail_df[col].apply(convertdatatype)

    df_string = avail_df.to_string(index=False)

    return df_string

def response(inputdate=''):
    data, datenow, exec_time = gettable(inputdate)
    returnstring = f"{datenow.strftime('%d %b %y, %A')}\n\n{condense(data)}\n\nfetch time: {str(exec_time)}s"
    return returnstring

def convertdatatype(input):
    try:
        return int(input)
    except:
        return str(input)
