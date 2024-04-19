import socket
from constants import HOST, PORT

client_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
# client_socket.connect(('91.236.168.95', 16158))
client_socket.connect((HOST, PORT))
try:
    while True:
        message_length = int.from_bytes(client_socket.recv(4), byteorder='big')
        response = client_socket.recv(message_length).decode('utf-8')
        message_type, message = response.split('$_$_$')
        print(message)
        if message_type == '1':
            while True:
                response = input()
                if 'Choose you name' in message or ('What is your bet?' in message and response == 'pass'):
                    break
                try:
                    int(response)
                    break
                except:
                    print('Invalid. try again')
            client_socket.sendall(response.encode())
except KeyboardInterrupt:
    print('Client is exiting...')
finally:
    client_socket.close()