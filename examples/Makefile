GN := ../target/release/gamlenorge

tifs := $(patsubst %.ini,%.tif,$(wildcard *.ini))

all: $(tifs)

clean:
	rm *.tif *~

%.tif: %.ini
	$(GN) -c $< --output $@

.PHONY: all clean
