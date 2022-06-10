'''liblog, a simple logging package'''

import __main__
import logging, sys, os
import inspect
import functools
import time

import modules.settings as s

logging.basicConfig(
    stream = sys.stdout,
    level = s.json.logger.log_level,
    format='%(levelname)-8s %(asctime)s %(message)s', #{%(module)s}:[%(funcName)s]
    datefmt='%Y-%m-%d %H:%M:%S'
    )

_liblogger = logging.getLogger()
'''Logger object. For internal use'''

## log format that resides within logging.basicConfig's 'message'
# _logformat = '{{{module}}}:[{function}] {message}'

def configure(*args, **kwargs):
    global _liblogger

    logging.basicConfig(*args, **kwargs, force=True)
    _liblogger = logging.getLogger()


class functions():
    '''Standard logger functions'''

    @staticmethod
    def debug(message:str=''):
        '''Log a debug message'''
        frame = inspect.stack()[1]
        function = frame.function
        module = os.path.splitext(os.path.basename(frame[1]))[0]
        _liblogger.debug(f'{{{module}}}:[{function}] {message}')
        return

    @staticmethod
    def info(message:str=''):
        '''Log an info message'''
        frame = inspect.stack()[1]
        function = frame.function
        module = os.path.splitext(os.path.basename(frame[1]))[0]
        _liblogger.info(f'{{{module}}}:[{function}] {message}')
        return

    @staticmethod
    def warning(message:str=''):
        '''Log a warning message'''
        frame = inspect.stack()[1]
        function = frame.function
        module = os.path.splitext(os.path.basename(frame[1]))[0]
        _liblogger.warning(f'{{{module}}}:[{function}] {message}')
        return

    @staticmethod
    def error(message:str=''):
        '''Log an error message'''
        frame = inspect.stack()[1]
        function = frame.function
        module = os.path.splitext(os.path.basename(frame[1]))[0]
        _liblogger.error(f'{{{module}}}:[{function}] {message}')
        return

    @staticmethod
    def critical(message:str=''):
        '''Log a critical message'''
        frame = inspect.stack()[1]
        function = frame.function
        module = os.path.splitext(os.path.basename(frame[1]))[0]
        _liblogger.critical(f'{{{module}}}:[{function}] {message}')
        return

class decorators():
    '''Logging decorator sub-class'''

    @staticmethod
    def debug(message:str=''):
        '''Decorator for logging a function entry: DEBUG level'''
        def inner(_function):
            @functools.wraps(_function)
            def wrapper(*args, **kwargs):
                if _function.__module__ == '__main__':
                    module = os.path.splitext(os.path.basename(__main__.__file__))[0]
                    function = _function.__name__
                else:
                    module = _function.__module__
                    function = _function.__name__
                _liblogger.debug(f'{{{module}}}:[{function}] {message}')

                return _function(*args, **kwargs)

            return wrapper
        return inner

    @staticmethod
    def info(message:str=''):
        '''Decorator for logging a function entry: INFO level'''
        def inner(_function):
            @functools.wraps(_function)
            def wrapper(*args, **kwargs):
                if _function.__module__ == '__main__':
                    module = os.path.splitext(os.path.basename(__main__.__file__))[0]
                    function = _function.__name__
                else:
                    module = _function.__module__
                    function = _function.__name__
                _liblogger.info(f'{{{module}}}:[{function}] {message}')

                return _function(*args, **kwargs)

            return wrapper
        return inner

    @staticmethod
    def warning(message:str=''):
        '''Decorator for logging a function entry: WARNING level'''
        def inner(_function):
            @functools.wraps(_function)
            def wrapper(*args, **kwargs):
                if _function.__module__ == '__main__':
                    module = os.path.splitext(os.path.basename(__main__.__file__))[0]
                    function = _function.__name__
                else:
                    module = _function.__module__
                    function = _function.__name__
                _liblogger.warning(f'{{{module}}}:[{function}] {message}')

                return _function(*args, **kwargs)

            return wrapper
        return inner

    @staticmethod
    def error(message:str=''):
        '''Decorator for logging a function entry: ERROR level'''
        def inner(_function):
            @functools.wraps(_function)
            def wrapper(*args, **kwargs):
                if _function.__module__ == '__main__':
                    module = os.path.splitext(os.path.basename(__main__.__file__))[0]
                    function = _function.__name__
                else:
                    module = _function.__module__
                    function = _function.__name__
                _liblogger.error(f'{{{module}}}:[{function}] {message}')

                return _function(*args, **kwargs)

            return wrapper
        return inner

    @staticmethod
    def critical(message:str=''):
        '''Decorator for logging a function entry: CRITICAL level'''
        def inner(_function):
            @functools.wraps(_function)
            def wrapper(*args, **kwargs):
                if _function.__module__ == '__main__':
                    module = os.path.splitext(os.path.basename(__main__.__file__))[0]
                    function = _function.__name__
                else:
                    module = _function.__module__
                    function = _function.__name__
                _liblogger.critical(f'{{{module}}}:[{function}] {message}')

                return _function(*args, **kwargs)

            return wrapper
        return inner

    class timers():
        '''Timing decorators sub-class'''

        @staticmethod
        def debug(message:str=''):
            '''Decorator for logging function execution time: DEBUG level'''
            def inner(_function):
                @functools.wraps(_function)
                def wrapper(*args, **kwargs):
                    t_start = time.time()
                    return_val = _function(*args, **kwargs)
                    t_end = time.time()

                    if _function.__module__ == '__main__':
                        module = os.path.splitext(os.path.basename(__main__.__file__))[0]
                        function = _function.__name__
                    else:
                        module = _function.__module__
                        function = _function.__name__
                    _liblogger.debug(f'{{{module}}}:[{function}]:[{(t_end - t_start):.6f}s]')

                    return return_val

                return wrapper
            return inner

