#include <door.h>
#include <err.h>
#include <fcntl.h>
#include <pwd.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stropts.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/uio.h>
#include <sys/un.h>
#include <unistd.h>

int stream_recv_fd(int sender) {
    struct strrecvfd data;
    ioctl(sender, I_RECVFD, &data);
    return data.fd;
}

int stream_send_fd(int receiver, int payload) {
    return ioctl(receiver, I_SENDFD, payload);
}

static int door_cache[256];

void target(
        void* cookie,
        char* argp, size_t arg_size,
        door_desc_t* dp, uint_t n_desc
    ) {
    FILE* f = fopen("README.md","r");
    char buffer[1024];
    fread(buffer, 1, 1024, f);
    printf("In the target sp running as uid=%d.\n", getuid());
    door_return(buffer, strlen(buffer) + 1, NULL, 0);
}

void proxy(
        void* cookie,
        char* argp, size_t arg_size,
        door_desc_t* dp, uint_t n_desc
    ) {
    printf("In the proxy sp.\n");
    char* data = "Hello";

    char* username = argp;
    struct passwd user;
    char userbuf[1024];
    getpwnam_r(username, &user, userbuf, 1024);
    
    if (door_cache[user.pw_uid] != 0) {
        printf("Reusing entry from cache\n");

        door_desc_t w;
        w.d_attributes = DOOR_DESCRIPTOR;
        w.d_data.d_desc.d_descriptor = door_cache[user.pw_uid];

        door_return(NULL, 0, &w, 1);
    }

    int stream[2];
    const int child = 0;
    const int parent = 1;
    pipe(stream);
    

    int pid = fork();
    if (pid == child) { // Child
        close(stream[parent]);
        setgid(user.pw_uid);
        setuid(user.pw_gid);
        chdir(user.pw_dir);

        int door_fd = door_create(target, NULL, 0);

	if (stream_send_fd(stream[child], door_fd) == -1)
	    err(1, "[child] stream_send_fd() failed");

	door_return(NULL,0,NULL,0);
    } else { // Parent
        close(stream[child]);

        int door_fd = stream_recv_fd(stream[parent]);
        if (door_fd == -1) {
            err(1, "[parent] stream_recv_fd() failed");
        }

        door_desc_t w;
        w.d_attributes = DOOR_DESCRIPTOR;
        w.d_data.d_desc.d_descriptor = door_fd;

        door_cache[user.pw_uid] = door_fd;
        door_return(data, 6, &w, 1);
    }
}

int main(int argc, char** argv) {
    // Require four arguments:
    // * --pid flag
    // * path to the pid storage file
    // * --door flag
    // * path to the server door
    if (argc < 5) {
        fprintf(stderr, "--pid,--door args missing\n");
        return 1;
    }
    // First argument must literally be '--pid' flag
    if (strncmp("--pid", argv[1], 5) != 0) {
        fprintf(stderr, "--pid opt missing\n");
        return 1;
    }
    // Second argument is therefore the pid path.
    char* pid_path = argv[2];
    // Third argument must literally be '--door' flag
    if (strncmp("--door", argv[3], 6) != 0) {
        fprintf(stderr, "--door opt missing\n");
        return 1;
    }
    // Fourth argument is therefore the door path.
    char* door_path = argv[4];

    // daemon
    daemon(1,1);

    // Write the current pid to the pid path
    FILE* pid_file = fopen(pid_path, "w");
    fprintf(pid_file, "%d\n", getpid());
    fclose(pid_file);

    // Spawn a proxy at the door path
    int door_fd = door_create(proxy, NULL, 0);
    creat(door_path, 0644);
    if (fattach(door_fd, door_path) != 0) {
        err(1, "Couldn't fattach");
    }

    return door_return(NULL, 0, NULL, 0);
}
