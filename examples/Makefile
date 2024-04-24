GN := ../target/release/gamlenorge
CONVERT := /usr/bin/convert

jpgs := $(patsubst %.ini,%.jpg,$(wildcard *.ini))

all: $(jpgs)

clean:
	rm -f *.tif *~

%.tif: %.ini
	$(GN) -c $< --output $@

%.jpg: %.tif
	$(CONVERT) $< $@

.PHONY: all clean
