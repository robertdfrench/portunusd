#include <door.h>
#include <stdio.h>
#include <unistd.h>
#include <fcntl.h>
#include <sys/stat.h>
#include <strings.h>
#include <err.h>

char handle_request(int client_fd) {
	// Get the client's name from the request
	char name[32];

	ssize_t name_len = read(client_fd, name, 31);
	if (name_len == -1) return 1; // Error 1: Client read failure.

	name[name_len] = 0; // Append a null byte so the string is proper.

	// Greet the client
	char greeting[64];

	int greeting_len = snprintf(greeting, 64, "Hello %s!", name);
	if (greeting_len < 0) return 2; // Error 2: Formatting failure.

	ssize_t bytes_written = write(client_fd, greeting, greeting_len);
	if (bytes_written == -1) return 3; // Error 3: Client write failure.

	return 0;
}

void answer_door(void* cookie, char* args, size_t nargs,
		door_desc_t* descriptors, uint_t ndescriptors) {
	// Error 4: No client file descriptor was provided
	char error_4 = 4;
	if (ndescriptors == 0) door_return(&error_4, 1, NULL, 0);

	int client_fd = descriptors[0].d_data.d_desc.d_descriptor;

	char rc = handle_request(client_fd);
	door_return(&rc, 1, NULL, 0);
}

void publish_pid(char* path) {
	FILE* f = fopen(path, "w");
	if (f == NULL) err(1, "Could not create pid file on disk");

	// The manpage says getpid will not fail, so no error handling is
	// needed in this case.
	pid_t pid = getpid();

	if (fprintf(f, "%d", pid) < 0) err(1, "Could not write pid to disk");
	if (fclose(f) != 0) err(1, "Could not close pid file after writing");
}

int main() {
	publish_pid("/var/run/hello_web.pid");
	char* path = "/var/run/hello_web_door";

	int door = door_create(&answer_door, NULL, 0);
	if (door == -1) err(1, "Handle cannot be attached to door");

	int fd = open(path, O_RDWR|O_CREAT|O_EXCL, 0400);
	if (fd < 0) err(1, "Could not create a new file for the door");

	int attachment = fattach(door, path);
	if (attachment == -1) err(1, "Could not attach door to filesystem");

	return door_return(NULL, 0, NULL, 0);
}
