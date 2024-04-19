HOST = 'localhost'
PORT = 12345
TYPES = [
    {
        'name': 'Spades',
        'unicode_char': '\u2660',
    },
    {
        'name': 'Hearts',
        'unicode_char': '\u2665',
    },
    {
        'name': 'Diamonds',
        'unicode_char': '\u2666',
    },
    {
        'name': 'Clubs',
        'unicode_char': '\u2663',
    }
]
HOKMS = [
    {
        'name': 'Spades',
        'unicode_char': '\u2660',
        'code': 0
    },
    {
        'name': 'Hearts',
        'unicode_char': '\u2665',
        'code': 1
    },
    {
        'name': 'Diamonds',
        'unicode_char': '\u2666',
        'code': 2
    },
    {
        'name': 'Clubs',
        'unicode_char': '\u2663',
        'code': 3
    }
]
ADDITIONAL_HOKMS = [
    {
        'name': 'Naras',
        'unicode_char': '\u2193',
        'code': 4
    },
    {
        'name': 'Saras',
        'unicode_char': '\u2191',
        'code': 5
    },
    {
        'name': 'Tak Naras',
        'unicode_char': '\u21A7',
        'code': 6
    }
]
NUMBERS = ['2', '3', '4', '5', '6', '7', '8', '9', '10', 'J', 'Q', 'K', 'A']
NUMBER_OF_PLAYERS = 4
TEAM_SIZE = 2
NUMBER_OF_TEAMS = NUMBER_OF_PLAYERS // TEAM_SIZE
INVALID_RESPONSE = 'Invalid. try again\n'