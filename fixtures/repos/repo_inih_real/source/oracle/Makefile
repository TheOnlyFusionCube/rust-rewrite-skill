CC ?= cc
CFLAGS ?= -O2 -Wall -Wextra
DEFINES = -DINI_MAX_LINE=1024 -DINI_MAX_SECTION=256 -DINI_MAX_NAME=256
# Keep other inih defaults: MULTILINE=1 BOM=1 INLINE=1 NO_VALUE=0 STOP=0

oracle_dump: oracle_dump.c ../upstream/ini.c ../upstream/ini.h
	$(CC) $(CFLAGS) $(DEFINES) -o $@ oracle_dump.c ../upstream/ini.c

clean:
	rm -f oracle_dump
