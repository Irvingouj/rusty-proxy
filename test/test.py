import socket
import ssl

# Initialize a socket
sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
sock.connect(('localhost', 3000))  # Connect to your proxy

# Send a CONNECT request
connect_req = b'CONNECT www.google.com:443 HTTP/1.1\r\nHost: www.google.com:443\r\n\r\n'
sock.sendall(connect_req)

# Receive the response
data = sock.recv(1024)
print(f"Received: {data.decode()}")

if b'200 Connection Established' in data:
    # SSL wrap the socket, then you can use it like a regular SSL socket
    context = ssl.SSLContext(ssl.PROTOCOL_TLS)
    ssl_sock = context.wrap_socket(sock, server_hostname='www.google.com')
    
    # Send an HTTPS request
    https_request = b'GET / HTTP/1.1\r\nHost: www.google.com\r\n\r\n'
    ssl_sock.sendall(https_request)

    # Receive some data from Google
    data = ssl_sock.recv(1024)
    print(f"Received: {data.decode()}")

    # Close the sockets
    ssl_sock.close()
else:
    print("Failed to establish connection through proxy")

sock.close()
