fn generate_world(
    mut commands: Commands,
    // Add other necessary resources, such as noise generators, biome data, etc.
) {
    // Create the GeoRegions
    let regions = generate_regions(/* ... */);

    for region in regions {
        // Create a region entity
        commands.spawn()
            .insert(GeoRegion { /* ... */ })
            .insert(EntityType::Region);

        for tile in region.tiles {
            // Spawn the tile entities within the region
            commands.spawn()
                .insert_bundle(TileBundle {
                    position: Position { /* ... */ },
                    height: Height { /* ... */ },
                    roughness: Roughness { /* ... */ },
                    // ... other components
                })
                .insert(EntityType::Tile);
        }
    }
}
