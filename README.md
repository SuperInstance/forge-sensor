# forge-sensor

> Sensor readings → tiles → Plato agents.

Decomposes raw sensor data (temperature, sonar, GPS, engine RPM, depth, wind, etc.) into timestamped tiles that Plato agents can read, filter, and transform.

Built for the fishing vessel demo — every sensor reading becomes a tile, every tile becomes a tick, every tick reaches an agent in the right room.

## What This Gives You

- **Parse any sensor line** — `"sonar:45.2:fathoms@1700001000"` → structured tile
- **Filter by type or time** — give me all sonar readings from the last hour
- **Latest by type** — what's the current reading for each sensor?
- **Statistics** — min, max, mean, std_dev for any set of readings
- **Zero dependencies** beyond serde + uuid

## Part of the ForgeFlux ecosystem

ForgeFlux decomposes any input into tiles. ForgeSensor decomposes sensor data. Plato agents subscribe to tiles as ticks. The fleet is the graph.

Apache 2.0
