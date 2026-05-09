# Open City Pipeline

Goal: turn public geospatial data into streamed city tiles the Bevy runtime can load.

## Data Policy

Do not derive world geometry from Google Maps content. Use open datasets:

- Overture Maps for buildings, places, transportation, and base layers.
- OpenStreetMap for supplemental tags where ODbL obligations are acceptable.
- USGS, NOAA, TIGER/Line, and municipal open-data portals for terrain, parcels, LiDAR, transit, and civic detail.

## Runtime Shape

The engine should not become a pure voxel renderer for every city surface. Use a hybrid model:

- `CityTile`: map-derived roads, buildings, places, and gameplay metadata.
- Voxel chunks: destructible pieces, interiors, terrain edits, and debug tools.
- Mesh assets: roads, sidewalks, facades, props, and low-detail distant city geometry.
- Graph assets: lanes, intersections, sidewalks, traffic spawn points, and nav nodes.

## First Vertical Slice

1. Generate a deterministic demo `CityTile`.
2. Convert the tile into Bevy meshes for roads and extruded buildings.
3. Stream a 3x3 tile neighborhood around the player.
4. Replace demo generation with an Overture/OSM preprocessing step.
5. Add road graph traversal for vehicles and NPCs.

## Public Data ETL Target

Preprocess outside the game loop:

```text
GeoParquet / OSM / DEM
  -> clip by bbox
  -> project to local meters
  -> simplify and normalize features
  -> split into CityTile ids
  -> emit compact tile blobs + mesh-ready metadata
```

The runtime should only load compact local tile blobs, not parse raw national-scale data.
