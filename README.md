# gamlenorge

Norwegian landscape rendering.

## Usage

  * Build code:
    cargo build release
  * Download geotiff maps from https://hoydedata.no/LaserInnsyn2
    (select 'export' -> 'landsdekkende')
  * Index maps in each zip file:
    ./target/release/index /my/geodata/maps/file.zip
  * Update configuration (use ./gamlenorge.ini as template)
  * Render landscape:
    ./target/release/gamlenorge -c mylandscape.ini

## Configuration

  The complete set of configuration is currently found in the source:

  ./src/config.rs
