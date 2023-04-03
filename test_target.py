#! /usr/bin/env python3

import sys
from threading import Thread
from socketserver import ThreadingMixIn
from http.server import HTTPServer, BaseHTTPRequestHandler


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200)
        self.send_header("Content-type", "text/plain")
        self.end_headers()
        self.wfile.write(bytes("ok", "utf-8"))


class ThreadingHTTPServer(ThreadingMixIn, HTTPServer):
    daemon_threads = True


def serve_on_port(port):
    server = ThreadingHTTPServer(("localhost", port), Handler)
    server.serve_forever()


if __name__ == '__main__':
    if len(sys.argv) != 3:
        print('usage: test_target.py MIN_PORT MAX_PORT')
        exit(1)
    min_port = int(sys.argv[1])
    max_port = int(sys.argv[2])
    if min_port < 1000 or max_port < 1000 or min_port > 65535 or max_port > 65535 or max_port < min_port:
        raise ValueError('bad ports')
    if max_port - min_port > 1000:
        raise ValueError('too many ports')
    print(f"min_port={min_port}, max_port={max_port}")
    threads = []
    for port in range(min_port, max_port):
        thread = Thread(target=serve_on_port, args=[port])
        threads.append(thread)
        thread.start()
    for thread in threads:
        thread.join()
