import os
import socket
import struct
from dotenv import load_dotenv
import msgpack
from dataclasses import dataclass
from enum import Enum


global player_cards, ground_cards, cur_bet, hokm


class Hokm(Enum):
    SPADES = "spades"
    HEARTS = "hearts"
    DIAMONDS = "diamonds"
    CLUBS = "clubs"
    NARAS = "naras"
    SARAS = "saras"
    TAK_NARAS = "tak_naras"
    DEFAULT = "default"

    def name_str(self) -> str:
        name_map = {
            Hokm.SPADES: "Spades",
            Hokm.HEARTS: "Hearts",
            Hokm.DIAMONDS: "Diamonds",
            Hokm.CLUBS: "Clubs",
            Hokm.NARAS: "Naras",
            Hokm.SARAS: "Saras",
            Hokm.TAK_NARAS: "Tak Naras",
            Hokm.DEFAULT: "Hokm",
        }
        return name_map[self]

    def unicode_char(self) -> str:
        unicode_map = {
            Hokm.SPADES: "\u2660",
            Hokm.HEARTS: "\u2665",
            Hokm.DIAMONDS: "\u2666",
            Hokm.CLUBS: "\u2663",
            Hokm.NARAS: "\u2193",
            Hokm.SARAS: "\u2191",
            Hokm.TAK_NARAS: "\u21a7",
            Hokm.DEFAULT: "",
        }
        return unicode_map[self]

    def code(self) -> str:
        code_map = {
            Hokm.SPADES: "S",
            Hokm.HEARTS: "H",
            Hokm.DIAMONDS: "D",
            Hokm.CLUBS: "C",
            Hokm.NARAS: "N",
            Hokm.SARAS: "A",
            Hokm.TAK_NARAS: "T",
            Hokm.DEFAULT: "",
        }
        return code_map[self]

    @classmethod
    def from_code(cls, code: str) -> "Hokm":
        code_to_hokm = {
            "S": cls.SPADES,
            "H": cls.HEARTS,
            "D": cls.DIAMONDS,
            "C": cls.CLUBS,
            "N": cls.NARAS,
            "A": cls.SARAS,
            "T": cls.TAK_NARAS,
        }
        return code_to_hokm.get(code, cls.DEFAULT)

    @classmethod
    def default(cls) -> "Hokm":
        return cls.DEFAULT

    def __str__(self) -> str:
        return f"{self.name_str()} {self.unicode_char()}"

    def __repr__(self) -> str:
        return f"Hokm.{self.name}"


@dataclass
class Card:
    type: Hokm
    number: str
    ord: int

    def code(self):
        return f"{self.type.code()}-{self.number}"

    @classmethod
    def from_code(cls, value: str) -> "Card":
        hokm_code, card_number = value.split("-", 1)
        hokm = Hokm.from_code(hokm_code)
        card_ord = NUMBERS.index(card_number)
        return Card(hokm, card_number, card_ord)

    def __repr__(self):
        return f"{self.type.unicode_char()} {self.number}"

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


NUMBERS = ["2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K", "A"]


class GameMessage(Enum):
    HANDSHAKE = "Handshake"
    HANDSHAKE_RESPONSE = "HandshakeResponse"
    BROADCAST = "Broadcast"
    USERNAME = "Username"
    USERNAME_RESPONSE = "UsernameResponse"
    TEAM_CHOICE = "TeamChoice"
    TEAM_CHOICE_RESPONSE = "TeamChoiceResponse"
    CARDS = "Cards"
    ADD_GROUND_CARDS = "AddGroundCards"
    BET = "Bet"
    FOLD = "Fold"
    HOKM = "Hokm"
    PLAY_CARD = "PlayCard"
    REMOVE_CARD = "RemoveCard"


class BroadcastMessage(Enum):
    GAME_STARTING = "GameStarting"
    HANDING_OUT_CARDS = "HandingOutCards"
    SHUFFLING_CARDS = "ShufflingCards"
    STARTER = "Starter"
    HOKM = "Hokm"
    BETS = "Bets"
    BET_WINNER = "BetWinner"
    GROUND_CARDS = "GroundCards"
    ROUND_WINNER = "RoundWinner"
    GAME_WINNER = "GameWinner"
    GAME_SCORE = "GameScore"
    ROUND_SCORE = "RoundScore"


def set_player_cards(response):
    global player_cards
    player_cards = [Card.from_code(card) for card in response["Cards"][0]]


def set_ground_cards(response):
    global ground_cards
    ground_cards = [
        (ground_card[0], Card.from_code(ground_card[1]))
        for ground_card in response["GroundCards"][0]
    ]


def set_hokm_broadcast(response):
    global hokm
    hokm = Hokm.from_code(response["Hokm"][0])


def set_bet(new_bet):
    global cur_bet
    cur_bet = new_bet


def print_hokm():
    print(f"Hokm: {hokm}")


def print_player_cards(indexed):
    print("These are your cards:")
    cards = [str(player_card) for player_card in player_cards]
    if indexed:
        cards = [
            f"{str(player_card)}: {i}" for i, player_card in enumerate(player_cards)
        ]
    print(", ".join(cards))


def print_ground_cards():
    if not ground_cards:
        return
    print("Played cards:")
    print(
        ", ".join(
            [f"{ground_card[0]}: {ground_card[1]}" for ground_card in ground_cards]
        )
    )


def receive_message(sock: socket.socket):
    try:
        length_data = sock.recv(4)
        if len(length_data) != 4:
            raise ConnectionError("Failed to read message length")
        message_length = struct.unpack(">I", length_data)[0]
        message_data = b""
        bytes_remaining = message_length
        while bytes_remaining > 0:
            chunk = sock.recv(min(bytes_remaining, 4096))
            if not chunk:
                raise ConnectionError("Connection closed while reading message")
            message_data += chunk
            bytes_remaining -= len(chunk)
        message = msgpack.unpackb(message_data, raw=False)
        return message
    except struct.error as e:
        raise ValueError(f"Failed to unpack message length: {e}")
    except msgpack.exceptions.ExtraData as e:
        raise ValueError(f"MessagePack deserialization error: {e}")
    except Exception as e:
        raise ConnectionError(f"Error receiving message: {e}")


def get_message_type(message) -> GameMessage:
    if isinstance(message, str):
        try:
            return GameMessage(message)
        except ValueError:
            raise ValueError(f"Unknown message type: {message}")
    elif isinstance(message, dict):
        if len(message) != 1:
            raise ValueError("Message should have exactly one key")
        message_type = list(message.keys())[0]
        try:
            return GameMessage(message_type)
        except ValueError:
            raise ValueError(f"Unknown message type: {message_type}")
    else:
        raise ValueError(f"Message should be string or dictionary, got {type(message)}")


def get_broadcast_message_type(message) -> GameMessage:
    if isinstance(message, str):
        try:
            return BroadcastMessage(message)
        except ValueError:
            raise ValueError(f"Unknown message type: {message}")
    elif isinstance(message, dict):
        if len(message) != 1:
            raise ValueError("Message should have exactly one key")
        message_type = list(message.keys())[0]
        try:
            return BroadcastMessage(message_type)
        except ValueError:
            raise ValueError(f"Unknown message type: {message_type}")
    else:
        raise ValueError(f"Message should be string or dictionary, got {type(message)}")


def send_message(sock: socket.socket, message_data) -> None:
    try:
        data = msgpack.packb(message_data)
        length = len(data)
        sock.sendall(struct.pack(">I", length))
        sock.sendall(data)
    except Exception as e:
        raise ConnectionError(f"Error sending message: {e}")


def team_choice(response, sock):
    available_teams = response["TeamChoice"][0]
    print(
        ", ".join(
            [
                f"{available_team}: {i}"
                for i, available_team in enumerate(available_teams)
            ]
        )
    )
    choice = choose(
        "Choose a team: ", response["TeamChoice"][1], len(available_teams) - 1, False
    )
    team_response = {"TeamChoiceResponse": {"team_index": choice}}
    send_message(sock, team_response)


def choose(prompt, server_error, max_value, passable):
    if server_error:
        print(f"Server error: {server_error}")
    while True:
        try:
            inpt = input(f"{prompt} (0-{max_value}): ")
            if inpt == "pass":
                if passable:
                    return inpt
                else:
                    print("You can't pass this one!")
                    continue
            choice = int(inpt)
            if 0 <= choice <= max_value:
                return choice
            else:
                print(f"Please enter a number from 0 to {max_value}")
        except ValueError:
            print("Please enter a valid number")


def username(sock):
    username = input("Enter your username: ")
    username_response = {"UsernameResponse": {"username": username}}
    send_message(sock, username_response)


def handshake(sock):
    send_message(sock, GameMessage.HANDSHAKE_RESPONSE.value)


def print_scores(scores):
    print(", ".join([f"{team_score[0]}: {team_score[1]}" for team_score in scores]))


def bet(response, sock):
    print_player_cards(False)
    passed = False
    choice = choose("what is your bet: ", response["Bet"][0], 13, True)
    if isinstance(choice, str):
        choice, passed = 0, True
    send_message(sock, create_player_choice(choice, passed))


def fold(response, sock):
    global player_cards
    print_player_cards(True)
    choice = choose(
        "Choose a card to fold: ", response["Fold"][0], len(player_cards) - 1, False
    )
    send_message(sock, create_player_choice(choice, False))


def set_hokm(response, sock):
    print_player_cards(False)
    hokms = [Hokm.SPADES, Hokm.HEARTS, Hokm.DIAMONDS, Hokm.CLUBS]
    if cur_bet == 13:
        hokms += [Hokm.NARAS, Hokm.SARAS, Hokm.TAK_NARAS]
    print(", ".join([f"{hokm}: {i}" for i, hokm in enumerate(hokms)]))
    choice = choose("What is your hokm? ", response["Hokm"][0], len(hokms) - 1, False)
    send_message(sock, create_player_choice(choice, False))


def create_player_choice(choice, passable):
    return {
        "PlayerChoice": {
            "index": choice,
            "passed": passable,
        }
    }


def sort_player_cards():
    global player_cards
    player_cards.sort(key=lambda card: (card.type.name_str(), card.ord))


def play_card(response, sock):
    global player_cards
    print_hokm()
    print_player_cards(True)
    print_ground_cards()
    prompt = "Choose a card to play: "
    while True:
        choice = choose(prompt, response["PlayCard"][0], len(player_cards) - 1, False)
        if ground_cards:
            ground = ground_cards[0][1]
            card = player_cards[choice]
            if (
                any(player_card.type == ground.type for player_card in player_cards)
                and card.type != ground.type
            ):
                if "You have " not in prompt:
                    prompt = f"You have {ground.type.name()}!\n{prompt}"
                continue
        break
    send_message(sock, create_player_choice(choice, False))


def add_ground_cards(response):
    global player_cards
    player_cards += [Card.from_code(card) for card in response["AddGroundCards"][0]]
    sort_player_cards()


def remove_card(response):
    global player_cards
    player_cards.remove(Card.from_code(response["RemoveCard"][0]))


def print_broadcast(response):
    response = response["Broadcast"][0]
    match get_broadcast_message_type(response):
        case BroadcastMessage.GAME_STARTING:
            print("All players connected. Game starting...!")
        case BroadcastMessage.HANDING_OUT_CARDS:
            print("Handing out cards...")
        case BroadcastMessage.SHUFFLING_CARDS:
            print("Shuffling cards...")
        case BroadcastMessage.STARTER:
            print(f"Starter: {response['Starter'][0]}")
        case BroadcastMessage.HOKM:
            set_hokm_broadcast(response)
            print_hokm()
        case BroadcastMessage.BETS:
            bets = []
            for bet in response["Bets"][0]:
                if isinstance(bet[1], str):
                    bets.append(f"{bet[0]}: {bet[1]}")
                else:
                    bets.append(f"{bet[0]}: {bet[1]['Choice']}")
            print(", ".join(bets))
        case BroadcastMessage.BET_WINNER:
            bet_winner = response["BetWinner"][0]
            set_bet(bet_winner[1])
            print(f"{bet_winner[0]} wins with {bet_winner[1]}")
        case BroadcastMessage.GROUND_CARDS:
            set_ground_cards(response)
            print_ground_cards()
        case BroadcastMessage.ROUND_WINNER:
            print(f"Winner of this round is: {response['RoundWinner'][0]}")
        case BroadcastMessage.GAME_WINNER:
            print(f"Game winner is: {response['RoundWinner'][0]}")
        case BroadcastMessage.ROUND_SCORE:
            print_scores(response["RoundScore"][0])
        case BroadcastMessage.GAME_SCORE:
            print_scores(response["GameScore"][0])
        case _:
            print(f"Pattern match: Unknown message type: {response}")


def main():
    global player_cards, ground_cards, cur_bet
    player_cards, ground_cards, cur_bet = [], [], 0
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    host = os.getenv("SERVER_HOST", "localhost")
    port = int(os.getenv("127.0.0.1", 12345))
    sock.connect((host, port))
    try:
        while True:
            response = receive_message(sock)
            match get_message_type(response):
                case GameMessage.HANDSHAKE:
                    handshake(sock)
                case GameMessage.BROADCAST:
                    print_broadcast(response)
                case GameMessage.USERNAME:
                    username(sock)
                case GameMessage.TEAM_CHOICE:
                    team_choice(response, sock)
                case GameMessage.CARDS:
                    set_player_cards(response)
                    print_player_cards(False)
                case GameMessage.BET:
                    bet(response, sock)
                case GameMessage.ADD_GROUND_CARDS:
                    add_ground_cards(response)
                    print_player_cards(False)
                case GameMessage.FOLD:
                    fold(response, sock)
                case GameMessage.HOKM:
                    set_hokm(response, sock)
                case GameMessage.PLAY_CARD:
                    play_card(response, sock)
                case GameMessage.REMOVE_CARD:
                    remove_card(response)
                case _:
                    print("Pattern match: Unknown message type")
    except Exception as e:
        print(f"Error: {e}")
    finally:
        sock.close()


if __name__ == "__main__":
    load_dotenv()
    main()
