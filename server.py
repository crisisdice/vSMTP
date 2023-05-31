from http.server import HTTPServer, BaseHTTPRequestHandler
import ssl

class SimpleHTTPRequestHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200)
        self.end_headers()

        with open('/root/www/index.html', 'rb') as index:
            self.wfile.write(index.read())

httpd = HTTPServer(('81.28.6.251', 443), SimpleHTTPRequestHandler)

httpd.socket = ssl.wrap_socket(
    httpd.socket,
    keyfile="/etc/vsmtp/certs/privkey.pem",
    certfile='/etc/vsmtp/certs/fullchain.pem',
    server_side=True)

httpd.serve_forever()
