#include <door.h>
#include <stdio.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <stdlib.h>
#include <err.h>
#include <sys/uio.h>
#include <unistd.h>
#include <string.h>
#include <syslog.h>

const unsigned short port = 8080;
const char ip4_address[16] = "0.0.0.0";
const char door_path[128] = "/var/run/hello_web_door";

int main() {
	int application = open(door_path, O_RDONLY);
	if (application == -1) err(1, "Could not open application door");

	int socket_fd = socket(AF_INET, SOCK_STREAM, 0);
	if (socket_fd == -1) err(1, "Could not open client connection");

	struct sockaddr_in in_sock;
	in_sock.sin_family = AF_INET;
	in_sock.sin_port = htons(port);
	int pton_rc = inet_pton(AF_INET, ip4_address, &in_sock.sin_addr);
	if (pton_rc != 1) err(1, "Could not convert ip string to network addr");

	int bind_rc = bind(socket_fd, (struct sockaddr*)&in_sock, sizeof(struct sockaddr_in));
	if (bind_rc == -1) err(1, "Could not bind socket to network address");

	int listen_rc = listen(socket_fd, 128);
	if (listen_rc == -1) err(1, "Could not begin listening to network");

	while(1) {
		struct sockaddr_in out_sock;
		socklen_t out_socklen = sizeof(struct sockaddr);
		int client_fd = accept(socket_fd, (struct sockaddr*)&out_sock, &out_socklen);
		if (client_fd == -1) err(1, "Could not accept client connection");

		// Prepare door args with client_fd
		door_desc_t w_descriptor;
		w_descriptor.d_attributes = DOOR_DESCRIPTOR;
		w_descriptor.d_data.d_desc.d_descriptor = client_fd;
		door_arg_t args = {0};
		args.desc_ptr = &w_descriptor;
		args.desc_num = 1;

		int result;
		result = door_call(application, &args);
		if (result == -1) err(1, "Could not invoke application via its door");

		result = close(client_fd);
		if (result == -1) err(1, "Could not terminate client");
	}


	return 0;
}
