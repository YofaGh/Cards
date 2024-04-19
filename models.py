class Card:
    def __init__(self, type_name, type_unicode_char, number, ord, type):
        self.type_unicode_char = type_unicode_char
        self.type_name = type_name
        self.number = number
        self.ord = ord
        self.type = type

    def __repr__(self):
        return f'{self.type_unicode_char} {self.number}'

    def __eq__(self, other):
        return self.type == other.type and self.ord == other.ord

    def __hash__(self):
        return hash((self.type, self.ord))

    def __lt__(self, other):
        return self.ord < other.ord

    def __gt__(self, other):
        return self.ord > other.ord

    def __le__(self, other):
        return self.ord <= other.ord

    def __ge__(self, other):
        return self.ord >= other.ord

    def __ne__(self, other):
        return not self.__eq__(other)

class Team:
    def __init__(self, name):
        self.name = name
        self.score = 0
        self.collected_hands = []
        self.players = []

    def __repr__(self):
        return str(self.players)

    def __eq__(self, other):
        return self.players == other.players

    def __hash__(self):
        return hash(self.players)

    def add_player(self, player):
        self.players.append(player)

class Player:
    def __init__(self, uuid, name, team, connection):
        self.name = name
        self.team = team
        self.uuid = uuid
        self.connection = connection

    def __repr__(self):
        return self.name

    def __eq__(self, other):
        return self.uuid == other.uuid

    def __hash__(self):
        return hash(self.uuid)

    def are_alias(self, other):
        return self.team == other.team

    def set_cards(self, cards):
        self.hand = sorted(cards, key=lambda card: (card.type_name, card.ord))

    def remove_card(self, card):
        self.hand.remove(card)

    def send_message(self, message, type=1):
        message = f'{type}$_$_${message}'.encode('utf-8')
        message_length = len(message).to_bytes(4, byteorder='big')
        self.connection.sendall(message_length + message)

    def recieve_message(self):
        return self.connection.recv(1024).decode()

class Ground:
    def __init__(self):
        self.hand = []

    def __repr__(self):
        return ', '.join([f'{card}:{player.name}' for player, card in self.hand])

    def __hash__(self):
        return hash((self.type, self.number))

    def add_card(self, player, card):
        self.hand.append((player, card))