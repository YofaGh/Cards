import socket
from constants import HOST, PORT

client_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
client_socket.connect((HOST, PORT))
try:
    while True:
        message_length = int.from_bytes(client_socket.recv(4), byteorder="big")
        response = client_socket.recv(message_length).decode("utf-8")
        if "$_$_$" in response:
            message_type, message = response.split("$_$_$")
        else:
            message_type, message = "0", ""
        print(message)
        if message_type == "1":
            while True:
                response = input()
                if "Choose your name" in message or (
                    "What is your bet?" in message and response == "pass"
                ):
                    break
                try:
                    int(response)
                    break
                except ValueError:
                    print("Invalid. try again")
            client_socket.sendall(response.encode())
except KeyboardInterrupt:
    print("Client is exiting...")
finally:
    client_socket.close()
