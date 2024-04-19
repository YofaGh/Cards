import threading, socket, random, uuid
from models import Player, Card, Team, Ground
from constants import *
global STARTER, HOKM

clients = []
lock = threading.Lock()

def generate_cards():
    global CARDS
    CARDS = []
    for TYPE in TYPES:
        for i in range(len(NUMBERS)):
            CARDS.append(Card(TYPE['name'], TYPE['unicode_char'], NUMBERS[i], i, TYPES.index(TYPE)))

def generate_teams():
    global TEAMS
    TEAMS = [Team(f'Team {i + 1}') for i in range(NUMBER_OF_TEAMS)]

def generate_field():
    global FIELD
    FIELD = []
    for i in range(TEAM_SIZE):
        for j in range(NUMBER_OF_TEAMS):
            FIELD.append(TEAMS[j].players[i])
    FIELD = tuple(FIELD)

def shuffle_cards():
    random.shuffle(CARDS)

def hand_out_cards():
    cards_per_player = len(CARDS) // NUMBER_OF_PLAYERS
    for i in range(NUMBER_OF_TEAMS):
        for j in range(TEAM_SIZE):
            TEAMS[i].players[j].set_cards(CARDS[cards_per_player * (i * TEAM_SIZE + j):(i * TEAM_SIZE + j) * cards_per_player + cards_per_player])

def set_starter(highest_beter, highest_bet):
    global STARTER
    if not STARTER or highest_bet == 13:
        STARTER = highest_beter
        return
    team_with_highest_score = max(team.score for team in TEAMS)
    if STARTER.team == team_with_highest_score:
        return
    STARTER = FIELD[FIELD.index(STARTER) + 1 % len(FIELD)]

def set_hokm(player, bet):
    global HOKM
    hokms = HOKMS
    if bet == 13:
        hokms += ADDITIONAL_HOKMS
    hokms_to_show = ', '.join([f'{hokm['unicode_char']} {hokm['name']}:{hokm['code']}' for hokm in hokms])
    pre = ''
    while True:
        player.send_message(f'{pre}{player.name} what is your hokm? {hokms_to_show}')
        hokm = int(player.recieve_message())
        if hokm > 3 and bet != 13:
            pre = INVALID_RESPONSE
            continue
        HOKM = HOKMS[hokm]
        break

def fold_first(player):
    folded_cards = []
    pre = ''
    while len(player.hand) > 12:
        player_hand = ', '.join([f'{player.hand[i]}:{i}' for i in range(len(player.hand))])
        player.send_message(f'{pre}{player_hand}\nChoose a card to fold')
        fold = int(player.recieve_message())
        if fold > len(player.hand) - 1:
            pre = INVALID_RESPONSE
            continue
        pre = ''
        folded_cards.append(player.hand[fold])
        del player.hand[fold]
    player.team.collected_hands.append(folded_cards)

def hand_collector(ground):
    if HOKM['code'] == 4:
        player_to_collect, min_card = ground.hand[0]
        for card in ground.hand[1:]:
            if card[1].type == ground.type and card[1].ord < min_card.ord:
                player_to_collect, min_card = card
    if HOKM['code'] == 5:
        player_to_collect, max_card = ground.hand[0]
        for card in ground.hand[1:]:
            if card[1].type == ground.type and card[1].ord > max_card.ord:
                player_to_collect, max_card = card
    if HOKM['code'] == 6:
        player_to_collect, min_card = ground.hand[0]
        for card in ground.hand[1:]:
            if card[1].type == ground.type and (card[1].ord < min_card.ord or card[1].type == '12'):
                player_to_collect, min_card = card
    else:
        player_to_collect, max_card = ground.hand[0]
        for card in ground.hand[1:]:
            if card[1].type == ground.type and (card[1].ord > max_card.ord):
                player_to_collect, max_card = card
        if ground.type != HOKM['code']:
            max_bor = None
            for card in ground.hand[1:]:
                if card[1].type == HOKM['code'] and (not max_bor or card[1].ord > max_bor.ord):
                    player_to_collect, max_bor = card
    return player_to_collect

def broadcast_message(message):
    for team in TEAMS:
        for player in team.players:
            player.send_message(message, 0)

def client_handler(connection):
    message = '1$_$_$Choose you name:'.encode('utf-8')
    message_length = len(message).to_bytes(4, byteorder='big')
    connection.sendall(message_length + message)
    name = connection.recv(1024).decode()
    pre = ''
    while True:
        available_teams = ', '.join([f'{team.name}:{i}' for i, team in enumerate(TEAMS) if len(team.players) < TEAM_SIZE])
        message = f'1$_$_${pre}Choose your team: {available_teams}'.encode('utf-8')
        message_length = len(message).to_bytes(4, byteorder='big')
        connection.sendall(message_length + message)
        reponse = connection.recv(1024).decode()
        team = TEAMS[int(reponse)]
        if len(team.players) >= TEAM_SIZE:
            pre = INVALID_RESPONSE
            continue
        player = Player(uuid.uuid4().hex, name, team, connection)
        team.add_player(player)
        clients.append(player)
        break

def start_game():
    global STARTER, CARDS
    STARTER = None
    generate_cards()
    generate_field()
    while all(team.score < 104 for team in TEAMS):
        broadcast_message('Shuffling cards...')
        shuffle_cards()
        broadcast_message('Handing out cards...')
        ground_cards = CARDS[:4]
        del CARDS[:4]
        hand_out_cards()
        highest_bet = None
        highest_beter = None
        for player in FIELD:
            player.send_message(f'These are your cards: {player.hand}\nWhat is your bet?')
            bet = player.recieve_message()
            if bet == 'pass':
                pass
            else:
                bet = int(bet)
                if not highest_bet or bet > highest_bet:
                    highest_bet = bet
                    highest_beter = player
                if highest_bet == 13:
                    break
        if not highest_bet:
            continue
        broadcast_message(f'{highest_beter.name} wins with {highest_bet}!')
        set_starter(highest_beter, highest_bet)
        highest_beter.set_cards(highest_beter.hand + ground_cards)
        broadcast_message(f'Starter: {STARTER.name}')
        fold_first(highest_beter)
        set_hokm(highest_beter, highest_bet)
        broadcast_message(f'Hokm: {HOKM["unicode_char"]} {HOKM["name"]}')
        round_starter = FIELD.index(STARTER)
        f_team, b_team = TEAMS[0], TEAMS[1]
        if highest_beter.team != f_team:
            f_team, b_team = b_team, f_team
        while len(f_team.collected_hands) < highest_bet or len(b_team.collected_hands) < (14 - highest_bet):
            broadcast_message(f'{f_team.name}: {len(f_team.collected_hands)}\n{b_team.name}: {len(b_team.collected_hands)}')
            ground = Ground()
            player_to_start = FIELD[round_starter]
            pre = ''
            while True:
                player_hand = ', '.join([f'{hand}:{i}' for i, hand in enumerate(player_to_start.hand)])
                player_to_start.send_message(f'{pre}{player_to_start.name}: {player_hand}\nChoose a card to play:')
                card = int(player_to_start.recieve_message())
                if card > len(player_to_start.hand) - 1:
                    pre = INVALID_RESPONSE
                    continue
                break
            card = player_to_start.hand[card]
            player_to_start.hand.remove(card)
            ground.add_card(player_to_start, card)
            ground.type = card.type
            ground.type_name = card.type_name
            for i in range(1, 4):
                broadcast_message(', '.join([f'{p_c[0].name}:{p_c[1]}' for p_c in ground.hand]))
                player_to_play = FIELD[(round_starter + i) % len(FIELD)]
                pre = ''
                while True:
                    player_hand = ', '.join([f'{hand}:{i}' for i, hand in enumerate(player_to_play.hand)])
                    player_to_play.send_message(f'{pre}{player_to_play.name}: {player_hand}\nChoose a card to play:')
                    card = int(player_to_play.recieve_message())
                    if card > len(player_to_play.hand) - 1:
                        pre = INVALID_RESPONSE
                        continue
                    card = player_to_play.hand[card]
                    if any(card.type == ground.type for card in player_to_play.hand) and card.type != ground.type:
                        pre = f'You have {ground.type_name}!\n'
                        continue
                    break
                player_to_play.hand.remove(card)
                ground.add_card(player_to_play, card)
            player_to_collect = hand_collector(ground)
            round_starter = FIELD.index(player_to_collect)
            player_to_collect.team.collected_hands.append(ground.hand)
        if len(f_team.collected_hands) == highest_bet:
            f_team.score += highest_bet if highest_bet != 13 else highest_bet * 2
        else:
            b_team.score += highest_bet * 2
        CARDS += ground_cards
        for TEAM in TEAMS:
            TEAM.collected_hands = []
    broadcast_message(f'Winner is {filter(lambda team: team.score >= 104, TEAMS)[0].name}!')

generate_teams()
server_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
server_socket.settimeout(0.2)
server_socket.bind((HOST, PORT))
server_socket.listen()

print(f'Server started on {HOST}:{PORT}')
try:
    while True:
        try:
            if len(clients) < NUMBER_OF_PLAYERS:
                client_socket, _ = server_socket.accept()
                thread = threading.Thread(target=client_handler, args=(client_socket,))
                thread.start()
                continue
            if len(clients) == NUMBER_OF_PLAYERS:
                broadcast_message('All players connected. Game starting...!')
                start_game()
                break
        except socket.timeout:
            pass
finally:
    server_socket.close()