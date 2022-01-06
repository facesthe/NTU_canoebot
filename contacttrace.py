import pandas as pd
import pandas as pd
import sheetscraper as ss

class tracer():
    def __init__(self):
        self.set = set([])

    def reset(self):
        self.__init__()

    def updateset(self, date):
        tempset = set(ss.namelist(date)[3:].tolist())

        if len(self.set) == 0: ## first case
            self.set |= tempset
        elif len(tempset & self.set) == 0:
            return
        else:
            self.set |= tempset

    def returntable(self):
       return pd.DataFrame(list(self.set), columns=[''])