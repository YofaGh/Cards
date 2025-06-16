import socket
from constants import HOST, PORT

client_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
client_socket.connect((HOST, PORT))

def recv_exact(sock, num_bytes):
    data = b""
    while len(data) < num_bytes:
        chunk = sock.recv(num_bytes - len(data))
        if not chunk:
            raise ConnectionError("Connection closed while reading data")
        data += chunk
    return data

try:
    while True:
        length_bytes = recv_exact(client_socket, 4)
        message_length = int.from_bytes(length_bytes, byteorder="big")
        message_data = recv_exact(client_socket, message_length)
        response = message_data.decode("utf-8")
        if "$_$_$" in response:
            message_type, message = response.split("$_$_$", 1)
        else:
            message_type, message = "0", response
        print(message)
        if message_type == "1":
            while True:
                user_response = input()
                if ("Choose your name" in message or 
                    "Enter your username" in message or 
                    ("What is your bet?" in message and user_response == "pass")):
                    break
                try:
                    int(user_response)
                    break
                except ValueError:
                    print("Invalid. try again")
            response_bytes = user_response.encode("utf-8")
            length_bytes = len(response_bytes).to_bytes(4, byteorder="big") 
            client_socket.sendall(length_bytes)
            client_socket.sendall(response_bytes)
except KeyboardInterrupt:
    print("Client is exiting...")
except ConnectionError as e:
    print(f"Connection error: {e}")
except Exception as e:
    print(f"Unexpected error: {e}")
finally:
    client_socket.close()
