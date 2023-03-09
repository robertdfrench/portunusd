#include <door.h>
#include <err.h>
#include <fcntl.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

int main(int argc, char** argv) {
    // Require two arguments:
    // * --first-door flag
    // * path to the first door
    if (argc < 3) {
        fprintf(stderr, "--first-door arg missing\n");
        return 1;
    }
    // First argument must literally be '--first-door' flag
    if (strncmp("--first-door", argv[1], 12) != 0) {
        fprintf(stderr, "--first-door opt missing\n");
        return 1;
    }
    // Second argument is therefore the first_door path.
    char* first_door_path = argv[2];

    // Call the first door
    int first_door = open(first_door_path, O_RDONLY);
    door_arg_t arg;
    arg.data_ptr = NULL;
    arg.data_size = 0;
    arg.desc_ptr = NULL;
    arg.desc_num = 0;
    arg.rbuf = NULL;
    arg.rsize = 0;
    door_call(first_door, &arg);
    close(first_door);
    printf("%d, %d, %d\n", arg.data_size, arg.desc_num, arg.rsize);
    if (arg.desc_num > 0) {
        door_desc_t* w = arg.desc_ptr;
        int second_door = w->d_data.d_desc.d_descriptor;
        door_call(second_door, NULL);
    }
    return 0;
}
