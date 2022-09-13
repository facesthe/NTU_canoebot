import unittest
from datetime import date, timedelta

import modules.formfiller as ff

class test_form_submit_duplicate_checker(unittest.TestCase):
    '''Put the submission checker (is_submitted_before) through a variety of tests'''


    def reset_global():
        '''Resets the formfiller global var'''
        ff.SUBMITTED_TIMES = [date.today() - timedelta(days=1) for i in range(2)]
        return


    def test_on_startup(self):
        '''tests that the checker passes on startup'''
        test_form_submit_duplicate_checker.reset_global()

        result_arr = [ff.is_submitted_before(date.today(), i) for i in range(2)]
        for result in result_arr:
            self.assertEqual(result, False)

        return


    def test_duplicate_submissions(self):
        '''test when encountering duplicate submissions on the same day'''

        test_form_submit_duplicate_checker.reset_global()

        ## run once
        garbage = [ff.is_submitted_before(date.today(), i) for i in range(2)]
        ## run twice
        result_arr = [ff.is_submitted_before(date.today(), i) for i in range(2)]
        for result in result_arr:
            self.assertEqual(result, True)

        return
