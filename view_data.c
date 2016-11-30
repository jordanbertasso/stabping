#include <stdio.h>
#include <stdint.h>

int main(int argc, char *argv[]) {
    if (argc != 2) {
        fprintf(stderr, "Please provide a stabping *.dat file as first argument\n");
    }

    char *filename = argv[1];
    FILE *file = fopen(filename, "r");

    int32_t buf[3];
    while (fread(buf, sizeof(int32_t), 3, file) == 3) {
        printf("%11d %2d %8d\n", buf[0], buf[1], buf[2]);
    }
}
