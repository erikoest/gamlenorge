# gamlenorge

Norwegian landscape rendering.

## Usage

  * Build code:
    cargo build release
  * Download geotiff maps from https://hoydedata.no/LaserInnsyn2
    (select 'export' -> 'landsdekkende')
  * Index maps in each zip file:
    ./target/release/index --maps /my/geodata/maps file.zip
  * Update configuration (use ./gamlenorge.ini as template)
  * Render landscape:
    ./target/release/gamlenorge -c mylandscape.ini

## Configuration parameters

Configuration parameters can be specified in the configuration file
./gamlenorge.ini, or a custom configuration specified on command line with
the -c / --config parameter.

The parameters in the configuration file can be overridden on command line
prefixing the parameter with a '--', e.g:

  gamlenorge -c custom.ini --haziness=1.5

### maps

Path to the maps directory. The directory is expected to contain atlas.json
files which contain references to tiff-files or zip-files containing tiff-files,
typically stored in the same directory. The atlas-files are created using the
command: index -m map_dir mapdata.zip

### observer

Coordinate of the observer. EU89 UTM33 coordinates are to be used, and the
format is N<northing>E<easting>. The observer height is the terrain level
plus observer_height_offset.

### target

Coordinate of the target. EU89 UTM33 coordinates are to be used, and the
format is N<northing>E<easting>. The target height is the terrain level plus
target_height_offset.

### observer_height_offset

Relative adjustment of observer position height level.

### target_height_offset

Relative adjustment of target position height level.

### width

Width of the rendered image.

### height

Height of the rendered image.

### width_angle

Angle from center of rendered image to horizontal edge.

### min_depth

Minimum depth at which to start rendering. The parameter is typically used to
increase performance if it is known that the area between observer and min_depth
is free from obstacles. It can also be used to 'see through' a landscape in
order to render the terrain behind it.

### max_depth

Maximum depth to render.

### haziness

Degree of atmospheric haziness.

### time

Time of the rendering. The time is used for calculating the position of the sun.

### output

Output filename of tiff image.
