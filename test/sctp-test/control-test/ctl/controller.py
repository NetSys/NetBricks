#!/usr/bin/env python
import socket
import sctp
def test():
    tcp = sctp.sctpsocket_tcp(socket.AF_INET)
    tcp.connect(('127.0.0.1', 8001))
    tcp.sctp_send("Hello")
if __name__ == "__main__":
    test()
