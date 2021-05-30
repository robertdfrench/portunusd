#include <door.h>
#include <stdio.h>
#include <sys/types.h>
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

	int client_fd = open("./junk_name", O_RDWR);
	if (client_fd == -1) err(1, "Could not open client connection");

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


	return 0;
}
