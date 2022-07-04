import unittest
from datetime import date

import modules.srcscraper as sc

class test_srcscraper_valid_codes(unittest.TestCase):
    '''Tests the validity of the facilities stored in root/.configs/srcscraper.config.json.'''

    def test_valid_facilities(self):
        '''Mass test existing facility table.'''
        for facil in range(len(sc.FACILITY_TABLE)):
            with self.subTest(facil = facil):
                self.assertFalse(
                    self.check_valid__booking_table_url(facil),
                    f"Check for valid url for facility number {facil}: in root/.configs/srcscraper.config.json"
                )

            # self.check_valid__booking_table_url(facil)
        return

    def check_valid__booking_table_url(self, facility_no:int):
        '''Test if the url presented returns an exception. (404 error)'''
        exception_raised = False

        try:
            sc.get_booking_table(date.today(), facility_no)
        except Exception as e:
            exception_raised = True

        return exception_raised
