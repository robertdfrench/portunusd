#include <door.h>
#include <err.h>
#include <fcntl.h>
#include <pwd.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stropts.h>

#define _XPG4_2
#include <sys/socket.h>
#undef  _XPG4_2

#include <sys/stat.h>
#include <sys/types.h>
#include <sys/uio.h>
#include <sys/un.h>
#include <unistd.h>

int sock_recv_fd(int sender) {
    char buffer[80];
    int payload;

    struct iovec iov[1];
    memset(iov, 0, sizeof(iov));
    iov[0].iov_base = buffer;
    iov[0].iov_len = sizeof(buffer);

    char cmsg_buf[CMSG_SPACE(sizeof(payload))];

    struct msghdr msg;
    msg.msg_name = NULL;
    msg.msg_namelen = 0;
    msg.msg_iov = iov;
    msg.msg_iovlen = 1;
    msg.msg_control = cmsg_buf;
    msg.msg_controllen = sizeof(cmsg_buf);
    msg.msg_flags = 0;
    if (recvmsg(sender, &msg, 0) == -1) {
        return -1;
    }

    struct cmsghdr *cmsg = CMSG_FIRSTHDR(&msg);
    unsigned char *cmsg_data = CMSG_DATA(cmsg);
    payload = *(int *)cmsg_data;

    return payload;
}

int sock_send_fd(int receiver, int payload) {
    char buffer[80];

    struct iovec iov[1];
    memset(iov, 0, sizeof(iov));
    iov[0].iov_base = buffer;
    iov[0].iov_len = sizeof(buffer);

    struct msghdr msg;
    msg.msg_name = NULL;
    msg.msg_namelen = 0;
    msg.msg_iov = iov;
    msg.msg_iovlen = 1;
    msg.msg_flags = 0;

    char cmsg_buf[CMSG_SPACE(sizeof(payload))];
    msg.msg_control = cmsg_buf;
    msg.msg_controllen = sizeof(cmsg_buf);

    struct cmsghdr *cmsg = CMSG_FIRSTHDR(&msg);
    cmsg->cmsg_len = CMSG_LEN(sizeof(payload));
    cmsg->cmsg_level = SOL_SOCKET;
    cmsg->cmsg_type = SCM_RIGHTS;

    unsigned char *cmsg_data = CMSG_DATA(cmsg);
    *(int *)cmsg_data = payload;

    msg.msg_controllen = cmsg->cmsg_len;
    return sendmsg(receiver, &msg, 0);
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

    int sock[2];
    const int child = 0;
    const int parent = 1;
    socketpair( AF_UNIX, SOCK_STREAM, 0, sock);
    

    int pid = fork();
    if (pid == child) { // Child
        //close(sock[parent]);
        setgid(user.pw_uid);
        setuid(user.pw_gid);
        chdir(user.pw_dir);

        int door_fd = door_create(target, NULL, 0);

	if (sock_send_fd(sock[child], door_fd) == -1)
	    err(1, "[child] sock_send_fd() failed");

	door_return(NULL,0,NULL,0);
    } else { // Parent
        //close(sock[child]);

        int door_fd = sock_recv_fd(sock[parent]);
        if (door_fd == -1) {
            err(1, "[parent] sock_recv_fd() failed");
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
