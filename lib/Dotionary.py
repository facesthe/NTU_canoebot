'''Derived class from dictionary.\n
Allows for dot notation in accessing dictionary elements.'''

class Dotionary(dict):
    '''Shamelessly copied from here:\n
    https://dev.to/0xbf/use-dot-syntax-to-access-dictionary-key-python-tips-10ec'''
    def __getattr__(self, key):
        try:
            return self[key]
        except KeyError as k:
            raise AttributeError(k)

    def __setattr__(self, key, value):
        self[key] = value

    def __delattr__(self, key):
        try:
            del self[key]
        except KeyError as k:
            raise AttributeError(k)

    def __repr__(self):
        return '<Dotionary ' + dict.__repr__(self) + '>'

def to_dotionary(dictionary: dict) -> Dotionary:
    '''Recursively converts all subelements into dotionary\n
    Parameter: dictionary to convert
    Returns: equivalent dotionary object'''
    ## base case
    if type(dictionary) != dict:
        return dictionary

    dictionary = Dotionary(dictionary)

    for item in dictionary.keys():
        dictionary[item] = to_dotionary(dictionary[item])

    return dictionary

def to_dictionary(dotionary: Dotionary) -> dict:
    '''Recursively converts a dotionary back into dictionary'''
    ## base case
    if type(dotionary) != Dotionary:
        return dotionary

    dotionary = dict(dotionary)

    for item in dotionary.keys():
        dotionary[item] = to_dictionary(dotionary[item])

    return dotionary
