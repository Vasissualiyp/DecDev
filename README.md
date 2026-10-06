# DecDev

A catalog of reusable, declaratively-specified game-development *capabilities*
(movement, camera, combat, inventory, …). Think "npm registry, but for
game-mechanic specs" — not a game engine.

Each capability is one YAML file under [`components/`](components/), validated
against [`schema/component.schema.json`](schema/component.schema.json). Specs
are descriptive metadata only: DecDev stores and links to implementations, it
never hosts or runs them. No proprietary game source, assets, or extracted
data is ever included; `reference_games` are named inspirations only.

See [`docs/architecture.md`](docs/architecture.md) for the design rationale and
[`specs/`](specs/) for the schemas, layout, and build plan.

## Components

<!-- COMPONENTS:START -->
| Name | Category | Reference games | Spec |
| --- | --- | --- | --- |
| Arcade Car Handling | vehicle | Mario Kart, Need for Speed | [arcade-car-handling.yaml](components/arcade-car-handling.yaml) |
| Branching Dialogue Tree | dialogue | Disco Elysium, The Witcher 3 | [branching-dialogue-tree.yaml](components/branching-dialogue-tree.yaml) |
| Dungeon Room Procgen | procgen | The Binding of Isaac, Enter the Gungeon | [dungeon-room-procgen.yaml](components/dungeon-room-procgen.yaml) |
| Grid Inventory | inventory | Resident Evil 4, Escape from Tarkov | [grid-inventory.yaml](components/grid-inventory.yaml) |
| Hitscan Weapon | combat | Doom, Half-Life | [hitscan-weapon.yaml](components/hitscan-weapon.yaml) |
| Quake-style Strafe Movement | movement | Quake, Quake III Arena | [quake-strafe-movement.yaml](components/quake-strafe-movement.yaml) |
| Quest Log Tracker | quest | Skyrim, The Witcher 3 | [quest-log-tracker.yaml](components/quest-log-tracker.yaml) |
| State Machine Enemy AI | enemy-ai | Halo, F.E.A.R. | [state-machine-enemy-ai.yaml](components/state-machine-enemy-ai.yaml) |
| Third-person Orbit Camera | camera | Dark Souls, The Legend of Zelda Breath of the Wild | [third-person-orbit-camera.yaml](components/third-person-orbit-camera.yaml) |
| Verlet Rope Simulation | physics | Shadow of the Colossus, Just Cause | [verlet-rope-simulation.yaml](components/verlet-rope-simulation.yaml) |
| Wall-Run Movement | character-controller | Titanfall 2, Mirror's Edge | [wall-run-movement.yaml](components/wall-run-movement.yaml) |
<!-- COMPONENTS:END -->

## Contributing

Add a `<slug>.yaml` file under `components/`, following the format in
[`specs/01-capability-spec-format.md`](specs/01-capability-spec-format.md),
then validate it with the CLI (`decdev validate`, once `cli/` exists — see
[`specs/06-build-plan.md`](specs/06-build-plan.md)) before opening a PR.

## License

Specs are offered under their per-file `license` field (typically
`CC-BY-4.0`). Implementations live in external repositories under their own
licenses.
