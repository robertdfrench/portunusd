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
#include <thread.h>

typedef struct _relay_plan {
    char ip4_address[16];
    unsigned short port;
    char door_path[128];
} RelayPlan;

typedef struct _relay_link {
    int application_descriptor;
    int network_descriptor;
} RelayLink;

RelayLink establish_relay_link(RelayPlan rp) {
    int application_descriptor = open(rp.door_path, O_RDONLY);
    if (application_descriptor == -1) err(1, "Could not open application door");

    int network_descriptor = socket(AF_INET, SOCK_STREAM, 0);
    if (network_descriptor == -1) err(1, "Could not open client connection");

    struct sockaddr_in in_sock;
    in_sock.sin_family = AF_INET;
    in_sock.sin_port = htons(rp.port);
    int pton_rc = inet_pton(AF_INET, rp.ip4_address, &in_sock.sin_addr);
    if (pton_rc != 1) err(1, "Could not convert ip string to network addr");

    int bind_rc = bind(network_descriptor, (struct sockaddr*)&in_sock,
            sizeof(struct sockaddr_in));
    if (bind_rc == -1) err(1, "Could not bind socket to network address");

    int listen_rc = listen(network_descriptor, 128);
    if (listen_rc == -1) err(1, "Could not begin listening to network");

    RelayLink rl = { application_descriptor, network_descriptor };
    return rl;
}

void* relay_loop(void* link_ptr) {
    RelayLink rl = *((RelayLink*)link_ptr);

    while(1) {
        struct sockaddr_in out_sock;
        socklen_t out_socklen = sizeof(struct sockaddr);
        int client_fd = accept(rl.network_descriptor,
                (struct sockaddr*)&out_sock, &out_socklen);
        if (client_fd == -1) err(1, "Could not accept client connection");

        // Prepare door args with client_fd
        door_desc_t w_descriptor;
        w_descriptor.d_attributes = DOOR_DESCRIPTOR;
        w_descriptor.d_data.d_desc.d_descriptor = client_fd;
        door_arg_t args = {0};
        args.desc_ptr = &w_descriptor;
        args.desc_num = 1;

        int result;
        result = door_call(rl.application_descriptor, &args);
        if (result == -1) err(1, "Could not invoke application via its door");

        result = close(client_fd);
        if (result == -1) err(1, "Could not terminate client");
    }

    return NULL;
}

int main() {
    // Get this from argv or config file eventually
    const uint8_t num_links = 2;
    const RelayPlan plans[num_links] = {
        { "0.0.0.0", 8080, "/var/run/hello_web_door" },
        { "0.0.0.0", 1234, "/var/run/caasio_door" }
    };
    RelayLink links[num_links];

    // Establish each relay link
    for (int i = 0; i < num_links; i++) {
        links[i] = establish_relay_link(plans[i]);
    }


    thread_t threads[num_links];
    for (int i = 0; i < num_links; i++) {
        thr_create(NULL, 0, relay_loop, &links[i], 0, &threads[i]);
    }

    while (thr_join(0, NULL, NULL) == 0)
        continue;

    return 0;
}
