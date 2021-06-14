---
title: Example how to read a shapefile, convert it to a `geo` data structure and check for polygon-point intersection
---

The example program in `./src/main.rs`

1. reads the polygons in `./tests/data/polygons.shp` (ESRI Shapefile) and the points in `./tests/data/points.shp` (ESRI Shapefile)
2. Converts them to the `geo` data-structure
3. Checks which polygons contain which points and
4. Emit a message with the corresponding feature-ids.

**Requirements**

Enable shapefiles `geo-types`-feature

`Cargo.toml`

```toml
[dependencies]
shapefile = {version = "0.3.0", features = ["geo-types"]}
...
```
