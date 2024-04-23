# gamlenorge

Norwegian landscape rendering.

![Alt text](stegastein.jpg?raw=true "Example rendering from Stegastein in Sognefjorden")

## Usage

  * Build code:
    cargo build release
  * Download geotiff maps from https://hoydedata.no/LaserInnsyn2
    (select 'export' -> 'landsdekkende')
  * Index maps in each zip file:
    cd /my/geodata/maps
    for z in *.zip; do
      (...)/gamlenorge/target/release/index --maps . z
    done
  * Update configuration (use ./gamlenorge.ini as template)
  * Render landscape:
    ./target/release/gamlenorge -c mylandscape.ini

## Utilities

### Gamlenorge

The main program for rendering the beautiful old Norwegian mountain
landscapes. E.g.:

<pre>
gamlenorge -c mylandscape.ini --output mylandscape.tif
</pre>

### Index

Used for creating atlas-files from a zipped package of geotiff maps,
or a directory of geotiff files.

<pre>
index --maps &lt;mapdir&gt; &lt;zipfile&gt;
index --maps &lt;mapdir&gt; &lt;tiffdir&gt;
</pre>

### Sun

Shows the angles of the sun (altitude and azimuth) for a given position
and a given time. Useful for adjusting the time parameter in order to
get optimal lightning for a landscape.

<pre>
sun &lt;position&gt; &lt;time&gt;
</pre>

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

Coordinate of the observer. EU89 UTM33 coordinates must be used, the
format is N&lt;northing&gt;>E&lt;easting&gt;. The observer height is the terrain level
plus observer_height_offset.

### target

Coordinate of the target. EU89 UTM33 coordinates must be used, the
format is N&lt;northing&gt;E&lt;easting&gt;. The target height is the terrain level plus
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
