/* oracle_dump.c — dump inih handler events for golden fixtures.
 * Compile with: cc -O2 -DINI_MAX_LINE=1024 -DINI_MAX_SECTION=256 -DINI_MAX_NAME=256 \
 *   -o oracle_dump oracle_dump.c ../upstream/ini.c
 * Usage: oracle_dump <file.ini>   OR   oracle_dump -  (stdin)
 * Output:
 *   e=<error_line>
 *   one line per handler call:  S=<section>|N=<name>|V=<value>
 * Escaping: \\ -> \\\\, | -> \\|, \n -> \\n, \r -> \\r, \t -> \\t
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "../upstream/ini.h"

static void emit_escaped(const char *s) {
    if (!s) { fputs("(null)", stdout); return; }
    for (; *s; s++) {
        unsigned char c = (unsigned char)*s;
        if (c == '\\') fputs("\\\\", stdout);
        else if (c == '|') fputs("\\|", stdout);
        else if (c == '\n') fputs("\\n", stdout);
        else if (c == '\r') fputs("\\r", stdout);
        else if (c == '\t') fputs("\\t", stdout);
        else fputc(c, stdout);
    }
}

static int dumper(void *user, const char *section, const char *name,
                  const char *value) {
    (void)user;
    fputs("S=", stdout); emit_escaped(section);
    fputs("|N=", stdout); emit_escaped(name);
    fputs("|V=", stdout); emit_escaped(value);
    fputc('\n', stdout);
    return 1;
}

int main(int argc, char **argv) {
    int error;
    if (argc != 2) {
        fprintf(stderr, "usage: %s <file.ini>|-\n", argv[0]);
        return 2;
    }
    if (strcmp(argv[1], "-") == 0) {
        /* read all stdin into buffer */
        size_t cap = 4096, n = 0;
        char *buf = malloc(cap);
        int ch;
        if (!buf) return 2;
        while ((ch = fgetc(stdin)) != EOF) {
            if (n + 1 >= cap) {
                cap *= 2;
                char *nb = realloc(buf, cap);
                if (!nb) { free(buf); return 2; }
                buf = nb;
            }
            buf[n++] = (char)ch;
        }
        buf[n] = '\0';
        error = ini_parse_string(buf, dumper, NULL);
        free(buf);
    } else {
        error = ini_parse(argv[1], dumper, NULL);
    }
    printf("e=%d\n", error);
    return 0;
}
