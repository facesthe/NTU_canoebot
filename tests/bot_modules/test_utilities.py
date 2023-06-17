import unittest

import bot_modules.utilities as ut

verify_dict: dict = {
    "ANON_FUNCTION_1": False,
    "ANON_FUNCTION_2": False,
    "ANON_FUNCTION_3": True,
    "ANON_FUNCTION_4": True

}

class test_verify(unittest.TestCase):


    def test_verify_function_noexec(self):
        '''
        Tests that the function does not execute
        by looking up its own name in a given dictionary.
        '''

        @ut.verify_exec(verify_dict)
        def anon_function_1():
            self.fail("Function should not have executed.")
            return

        anon_function_1()
        return


    def test_verify_function_noexec_key(self):
        '''Tests that the functon does not execute
        by looking up a key passed as a parameter.
        '''

        @ut.verify_exec(verify_dict, "anon_function_2")
        def anon_function_2():
            self.fail("Function should not have executed.")
            return

        anon_function_2()
        return


    def test_verify_function_exec(self):
        '''
        Tests that the function executes by looking up
        its own name in the given dictionary.
        '''

        @ut.verify_exec(verify_dict)
        def anon_function_3():
            return True

        self.assertEqual(anon_function_3(), True, "Function should have executed.")


    def test_verify_function_exec_key(self):
        '''
        Tests that the function execeutes after looking up its
        given key in a given dictionary.
        '''

        @ut.verify_exec(verify_dict, "anon_function_4")
        def anon_function_4():
            return True

        self.assertEqual(anon_function_4(), True, "function should have executed.")


    def test_verify_function_exec_no_key(self):
        '''
        Tests that the function follows the default behavior if
        its name/key given does not exist in the given dictionary.
        '''

        @ut.verify_exec(verify_dict)
        def anon_function_5():
            return True

        if ut.VERIFY_MISSING_BEHAVIOR: ## if default is to exec
            self.assertEqual(anon_function_5(), True, "Function should execute by default.")
        else:   ## if default is not to exec
            self.assertIsNone(anon_function_5(), "function should not execute by default.")


