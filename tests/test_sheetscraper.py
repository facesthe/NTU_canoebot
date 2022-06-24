import unittest
from datetime import date

import modules.sheetscraper as ss

class test_sheetscraper(unittest.TestCase):
    months = [date(2022,i,3) for i in range(1,13)]

    def test_getlastsun(self):
        for i in range(12):
            self.assertEqual(
                ss.getlastsun(self.months[i]).weekday(),
                6,
                "Function should return a sunday"
            )

        self.assertEqual(
            ss.getlastsun(date(2022,1,1)).isocalendar(),
            date(2022,1,30).isocalendar(),
            "Last sunday of month should follow current month"
        )
        return

    def test_getfirstmon(self):
        for i in range(12):
            self.assertEqual(
                ss.getfirstmon(self.months[i]).weekday(),
                0,
                "Function should return a monday"
            )

        self.assertEqual(
            ss.getfirstmon(date(2022,1,1)).isocalendar(),
            date(2021,12,27).isocalendar(),
            "First monday of the month should be right after last sunday of previous month"
        )
        return

    def test_getsheetdate(self):
        sheet_dates = [
            date(2021,12,27),
            date(2022,1,31),
            date(2022,2,28),
            date(2022,3,28),
            date(2022,4,25),
            date(2022,5,30),
            date(2022,6,27),
            date(2022,8,1),
            date(2022,8,29),
            date(2022,9,26),
            date(2022,10,31),
            date(2022,11,28),
        ]

        for i in range(12):
            self.assertEqual(
                ss.getsheetdate(self.months[i]),
                sheet_dates[i],
                "Sheet date should be a monday right after the last sunday of previous month"
            )
        return
