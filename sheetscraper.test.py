from datetime import date, timedelta
from dateutil.parser import parse
import sheetscraper as ss
import pandas as pd

# testDate = parse("29 nov 2021").date()
# z = ss.namelist("29 nov 2021")
#print(z)

# x = ss.getsheetname(testDate)
# print(x)

## convert dataframe or series into string form
## justify all left
def string_df(df_in):
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


x = ss.getnamesv2('friday', 0)
print(x)

# # string_df(z)
# print(type(z) == pd.DataFrame)
# print(string_df(z))

# a = pd.DataFrame({
#     "col1": [1,2,3,4],
#     "col2": [5,6,7,8]
# })
# print(string_df(a))
# print(ss.df2str(ss.boatallo()))
