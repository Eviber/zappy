import socket
import sys
import threading

def handle_stdin(client_sock):
    """Read from stdin and send to client"""
    try:
        while True:
            data = sys.stdin.readline()
            if not data:
                break
            client_sock.sendall(data.encode())
    except Exception as e:
        print(f"Error sending stdin: {e}", file=sys.stderr)

def start_server(filename, host='localhost', port=9999):
    """Start TCP server that sends file contents then stdin"""
    server_sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server_sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    server_sock.bind((host, port))
    server_sock.listen(1)

    print(f"Server listening on {host}:{port}")

    try:
        while True:
            # Accept connection
            client_sock, addr = server_sock.accept()
            print(f"Client connected from {addr}")

            try:
                # Send file contents
                with open(filename, 'rb') as f:
                    while True:
                        chunk = f.read(4096)
                        if not chunk:
                            break
                        client_sock.sendall(chunk)
                print(f"File '{filename}' sent successfully")

                # Start redirecting stdin in a separate thread
                stdin_thread = threading.Thread(target=handle_stdin, args=(client_sock,))
                stdin_thread.daemon = True
                stdin_thread.start()

                # Keep connection alive
                stdin_thread.join()

            except FileNotFoundError:
                print(f"Error: File '{filename}' not found", file=sys.stderr)
            except Exception as e:
                print(f"Error: {e}", file=sys.stderr)
            finally:
                client_sock.close()
                print("Client disconnected, waiting for next connection...")

    except KeyboardInterrupt:
        print("\nServer shutting down")
    finally:
        server_sock.close()

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python server.py <filename> [host] [port]")
        sys.exit(1)

    filename = sys.argv[1]
    host = sys.argv[2] if len(sys.argv) > 2 else 'localhost'
    port = int(sys.argv[3]) if len(sys.argv) > 3 else 9999

    start_server(filename, host, port)
