'''Collection of functions that fetch, parse, format and store attendance data.'''
import pandas as pd
import numpy as np
import copy
from datetime import datetime, date, timedelta
from dateutil.relativedelta import relativedelta

import modules.sheetscraper as ss
import modules.settings as s

import lib.liblog as lg

NUM_WEEK_COLS = 14
WEEK_GAP_COLS = 3

def get_filler_cols_to_remove(num_weekday_cols:int, num_filler_cols:int, total_cols: int) -> "list[int]":
    '''Returns an array containing the indices of columns to be removed.
    Array is sorted in reverse order'''

    cols_to_remove: "list[int]" = []

    for i in range(total_cols // num_weekday_cols):
        for j in range(1, num_filler_cols + 1):
            index = (num_weekday_cols + num_filler_cols) * i + j

            if index < total_cols:
                cols_to_remove.append(index)
            else:
                continue

    cols_to_remove.sort(reverse=True)
    return cols_to_remove

def reset_df_indices(df_in:pd.DataFrame):
    '''Reset the row and column indices to an ordered list starting from 0'''
    df_in.reset_index(drop=True, inplace=True)
    df_in.columns = range(df_in.columns.size)

def format_sheet_df(df_in: pd.DataFrame) -> pd.DataFrame:
    '''formats a raw attendance sheet into a cleaner version'''

    return_df:pd.DataFrame = copy.deepcopy(df_in)

    ## remove redundant rows
    return_df = return_df.iloc[np.r_[1,3:ss.findlowestname(return_df)], :]
    ## remove filler cols
    return_df.drop(
    return_df.columns[
        get_filler_cols_to_remove(
            NUM_WEEK_COLS,
            WEEK_GAP_COLS,
            return_df.columns.size
        )],
    axis=1,
    inplace=True
    )

    ## drop the empty row separating guys and gals
    return_df.dropna(how="all", axis=0, inplace=True)

    ## replace dates with ISO date formats
    reset_df_indices(return_df)
    df_start_date: date = datetime.strptime(return_df.iloc[0,0], "%d-%b-%y").date()

    for i in range(1, return_df.columns.size, 2):
        return_df.iloc[0, [i, i+1]] = df_start_date.isoformat()
        df_start_date += timedelta(days=1)

    ## fill empty cells with "N", convert all fields to uppercase
    return_df.fillna("N", inplace=True)
    return_df = return_df.convert_dtypes()
    return_df = return_df.applymap(str.upper) ## apply function elementwise

    ## set col headers and indices
    return_df_header = return_df.iloc[0]
    return_df_header[0] = "Name"
    return_df = return_df[1:]
    return_df.columns = return_df_header

    return_df.set_index("Name", inplace=True)

    return return_df

def unweave_attendance_frames(df_in: pd.DataFrame) -> "tuple[pd.DataFrame, pd.DataFrame]":
    '''separates the 2 conjoined AM and PM dataframes, returning a tuple containing each frame.
    AM frame is stored in the first element, PM frame in the second element.'''

    working_df = copy.deepcopy(df_in)

    df_am:pd.DataFrame = working_df.iloc[:, [2 * i for i in range(working_df.columns.size // 2)]]
    df_pm: pd.DataFrame = working_df.iloc[:, [2 * i + 1 for i in range(working_df.columns.size // 2)]]

    return (df_am, df_pm)

def fetch_format_store_month_frames() -> date:
    '''Performs checks, updates stored database from Sheets. Main fetch function.
    Returns the last valid month and year in date format (ignore day)'''
    date_today = date.today()

    ## to prevent months that do not have >28 days to return an error
    start_date:date = date(date_today.year, date_today.month, 1)

    lg.functions.debug(start_date)

    while True: ## loop until a sheet is not found
        month_df:pd.DataFrame
        formatted_df:pd.DataFrame
        df_tuple: "tuple[pd.DataFrame, pd.DataFrame]"

        try:
            month_df = ss.getsheet(date(start_date.year, start_date.month, 1))
            lg.functions.debug(f'get sheet successful')
        except:
            ## stop on error in fetching oldest sheet
            return start_date + relativedelta(months=1)

        try:
            formatted_df = format_sheet_df(month_df)
        except:
            return start_date + relativedelta(months=1)

        df_tuple = unweave_attendance_frames(formatted_df)

        df_tuple[0].to_csv(f"./data/attendance/AM/{start_date.strftime('%Y-%m')}.csv")
        df_tuple[1].to_csv(f"./data/attendance/PM/{start_date.strftime('%Y-%m')}.csv")

        start_date -= relativedelta(months=1)


