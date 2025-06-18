from models import Hokm

HOST = "localhost"
PORT = 12345
SPADES = Hokm("Spades", "\u2660")
HEARTS = Hokm("Hearts", "\u2665")
DIAMONDS = Hokm("Diamonds", "\u2666")
CLUBS = Hokm("Clubs", "\u2663")
NARAS = Hokm("Naras", "\u2193")
SARAS = Hokm("Saras", "\u2191")
TAK_NARAS = Hokm("Tak Naras", "\u21a7")
TYPES = [SPADES, HEARTS, DIAMONDS, CLUBS]
HOKMS = [SPADES, HEARTS, DIAMONDS, CLUBS, NARAS, SARAS, TAK_NARAS]
NUMBERS = ["2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K", "A"]
NUMBER_OF_PLAYERS = 4
TEAM_SIZE = 2
NUMBER_OF_TEAMS = NUMBER_OF_PLAYERS // TEAM_SIZE
INVALID_RESPONSE = "Invalid. try again\n"
PROTOCOL_SEP = "$"
