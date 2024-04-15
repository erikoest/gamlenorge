# gamlenorge

Norwegian landscape rendering.

## Usage

o Build code:
  cargo build release
o Download geotiff maps from https://hoydedata.no/LaserInnsyn2
  (select 'export' -> 'landsdekkende')
o Index maps in each zip file:
  ./target/release/index /my/geodata/maps/file.zip
o Update configuration (use ./gamlenorge.ini as template)
o Render landscape:
  ./target/release/gamlenorge -c mylandscape.ini

## Configuration
  The complete set of configuration is currently found in the source:
  ./src/config.rs
