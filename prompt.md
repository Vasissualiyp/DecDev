I want you to act as a senior systems architect, game-engine architect, and startup technical founder.

I am exploring a new platform for AI-assisted game development based on the idea that game mechanics and systems should become **composable, reusable components**, similar to LEGO.

## Deliverable and success criteria

Produce two things, not one:

1. A condensed architecture proposal covering sections A–M below. "Condensed" means decisive — pick a position and defend it in a few sentences, don't survey every option at equal length. Section M (critique) is the part I care most about; don't let it become a short coda after twelve sections of enthusiasm.
2. A buildable MVP spec derived from (1): concrete enough (schemas, field names, file layout, commands) that a less capable coding agent could implement it without making architectural judgment calls of its own.

I will treat unresolved ambiguities in this brief as decisions for you to make and document, not blockers to hand back to me — flag assumptions, don't ask about every one of them.

## Core idea

Recent advances in game decompilation/recompilation and AI-assisted code analysis suggest that it is becoming increasingly feasible to analyze existing games, understand their mechanics and software architecture, and reproduce individual systems independently.

The long-term vision is NOT to redistribute copyrighted game code or assets.

Instead, the platform should allow developers and AI agents to discover, specify, implement, adapt, and compose **game-development capabilities** such as:

* Quake-like FPS movement
* Mario-like platforming
* Souls-like lock-on
* Doom-like weapon systems
* inventory systems
* dialogue systems
* quest systems
* procedural world generation
* camera systems
* vehicle controllers
* character controllers
* enemy AI
* animation systems
* physics behaviors
* rendering techniques
* networking/replication systems
* etc.

The reference game can provide behavioral inspiration or a specification, while the actual reusable implementation should be independently licensed/created where necessary.

The ultimate goal is a **"platform-less platform" for game development**:

> Instead of forcing developers into Unity, Unreal, Godot, etc., define a common set of machine-readable capabilities and interfaces. Individual implementations can be swapped, composed, or adapted by AI agents.

For example, a game might conceptually be:

```
Camera          ← Zelda-like implementation
Movement        ← Fallout-like implementation
Combat          ← Dark Souls-like implementation
Inventory       ← Diablo-like implementation
World generation← Minecraft-like implementation
```

The developer should be able to replace one component without rewriting the rest of the game.

## The key technical challenge

Existing game systems are often highly coupled.

"Movement" may directly interact with:

* physics
* animation
* camera
* input
* stamina
* audio
* networking
* character state
* collision
* AI
* scripting

Therefore, simply extracting source files is NOT sufficient.

The platform needs to define stable **capability interfaces/contracts**.

For example:

```
CharacterMovement
    Inputs:
        MovementInput
        CharacterState
        PhysicsState

    Outputs:
        Velocity
        MovementState

    Events:
        Jumped
        Landed
        StartedFalling
```

A component should communicate through standardized interfaces, semantic events, data structures, and contracts rather than directly depending on the internals of other components.

The platform should potentially have:

1. Primitive interfaces

   * Entity
   * Transform
   * Input
   * PhysicsBody
   * Animation
   * Audio
   * Camera
   * Resource
   * Event
   * Time

2. Higher-level capabilities

   * CharacterMovement
   * Combat
   * Inventory
   * Camera
   * Dialogue
   * Targeting
   * VehicleControl
   * Navigation
   * QuestSystem
   * ProceduralGeneration
   * etc.

3. Game-specific implementations of those capabilities.

4. AI-generated adapters when an implementation does not perfectly match an interface.

5. Machine-readable behavioral contracts describing:

   * inputs
   * outputs
   * units
   * state
   * events
   * dependencies
   * lifecycle/update ordering
   * invariants
   * performance requirements
   * determinism requirements
   * networking requirements
   * version compatibility

## Potential architectural analogy

Consider compiler architecture:

```
Source language
      ↓
Intermediate Representation
      ↓
Backend
      ↓
x86 / ARM / GPU
```

The platform could similarly use a **Game Capability Intermediate Representation (Game Capability IR)**:

```
Existing implementation / game analysis
      ↓
Behavioral specification
      ↓
Game Capability IR
      ↓
Component graph
      ↓
AI-generated adapters / implementations
      ↓
Target runtime / engine
```

The Game Capability IR may become the central abstraction of the entire platform.

It should ideally be possible to target:

* custom C++
* a custom runtime
* Unreal
* Unity
* Godot
* other engines
* potentially browser/WebGPU
* potentially other future runtimes

The platform should not necessarily become another monolithic game engine.

## AI agents

AI agents are a first-class consumer.

Eventually an agent should be able to query something like:

```
Find a third-person locomotion implementation
with:
    - high acceleration
    - low air control
    - slope handling
    - deterministic physics
```

Or:

```
Compose:
    Quake movement
    Souls lock-on
    Zelda camera
```

The agent should determine compatibility, resolve dependencies, generate adapters, modify the user's project, compile it, run tests, and produce a diff.

The platform should therefore expose:

* REST/GraphQL API or equivalent
* CLI
* MCP/tool interface for AI agents
* machine-readable component specifications
* dependency graph
* compatibility metadata
* versioning
* provenance
* licensing information
* automated tests
* benchmarks where relevant

## Human-facing website

The initial product should be a searchable web catalog.

Users should be able to browse/search components by:

* capability
* genre
* behavior
* reference game
* engine
* language
* license
* maturity
* dependencies
* performance
* compatibility

A component page might contain:

```
Name
Description
Behavioral specification
Reference games
Implementation(s)
License
Provenance
Dependencies
Interfaces
Events
Compatibility
Tests
Benchmarks
Demo/sandbox
GitHub/repository
"Add to project"
```

Eventually components should have interactive previews so developers can experience the behavior before installing it.

## Critical legal/product constraint

Do NOT design the platform around hosting or redistributing proprietary game source code, ROMs, extracted assets, textures, models, audio, etc.

We want to distinguish clearly between:

1. Behavioral ideas/mechanics
2. Behavioral specifications
3. Independently implemented code
4. Open-source implementations
5. Developer-contributed implementations
6. Clean-room/reimplementation workflows
7. Proprietary source material used only as analysis/reference material where legally permissible

The platform should have explicit provenance and licensing metadata.

Treat legal/IP questions carefully and flag areas requiring professional legal advice rather than assuming that reverse engineering automatically makes redistribution legal.

## MVP

The immediate goal is NOT to build the complete platform.

I want an MVP that could plausibly be built by one technically capable person using AI coding agents over a few evenings/a weekend.

The MVP could simply be:

* searchable web catalog
* component metadata
* tags/categories
* component pages
* GitHub/repository links
* licensing/provenance information
* basic API
* user submissions
* perhaps voting/bookmarking
* CLI for querying the catalog

Initially, components can be manually curated.

Do NOT over-engineer the first version. Concretely: no hosted code execution, no automated decompilation pipeline, and no attempt at runtime composition in the MVP — those are long-term-track problems (see sections D, E, G). If you're unsure whether something belongs in the MVP, the default answer is it doesn't.

Infrastructure budget for the MVP is effectively $0 — assume no paid hosting, no managed database unless genuinely free-tier. Prefer designs that get this for free (e.g. static hosting, data-as-files-in-git) over ones that are "simple" only until the bill arrives.

The architecture should, however, avoid making the eventual Game Capability IR impossible.

## Long-term goal

Eventually I want something where the user can say:

```
"Build me a third-person action RPG using:
   Fallout-style locomotion
   Dark Souls-style combat
   Zelda-style camera
   Diablo-style inventory
   Minecraft-style procedural terrain."
```

The AI agent should:

1. Search the component registry.
2. Interpret the requested behaviors.
3. Resolve implementations.
4. Construct a component dependency graph.
5. Detect incompatibilities.
6. Generate adapters.
7. Generate missing glue code.
8. Build the project.
9. Run automated tests.
10. Produce a working game project.
11. Allow individual components to be replaced without destroying unrelated systems.

The eventual system should feel like **LEGO for software**, rather than like an AI that writes an entire monolithic game from scratch.

## What I want from you

Produce a serious technical specification and architecture proposal.

Please cover:

### A. Product architecture

* MVP
* intermediate versions
* long-term platform
* major milestones
* what should NOT be built initially

### B. Game Capability IR

Design a concrete initial schema.

Show examples in JSON/YAML and/or strongly typed definitions.

Explain:

* capabilities
* interfaces
* data contracts
* semantic events
* dependencies
* lifecycle
* versioning
* compatibility
* invariants
* units
* state
* deterministic behavior

### C. Component model

Define what makes something a valid reusable component.

Explain how to deal with:

* tightly coupled legacy systems
* hidden dependencies
* global state
* update-order assumptions
* engine-specific APIs
* physics assumptions
* animation assumptions
* networking
* save/load
* performance constraints

### D. AI decomposition pipeline

Design a pipeline for taking an existing implementation and producing:

```
source/repository
    ↓
analysis
    ↓
dependency discovery
    ↓
behavioral specification
    ↓
capability classification
    ↓
Game Capability IR
    ↓
decoupled implementation
    ↓
tests
    ↓
published component
```

Explain what can realistically be automated by current AI coding agents and what still requires human engineering.

### E. Composition/adaptation

Design the system that allows:

```
Component A + Component B
```

when they were not designed to work together.

Explain:

* adapters
* type conversion
* semantic conversion
* event translation
* coordinate systems
* units
* lifecycle/update ordering
* conflicting assumptions
* dependency resolution
* automatic compatibility checking

### F. Agent interface

Design:

* CLI
* REST/API
* MCP interface
* agent workflow
* commands such as:

  search
  inspect
  install
  compose
  adapt
  test
  benchmark
  publish

Show concrete examples.

### G. Runtime strategy

Compare three approaches:

1. Build our own minimal runtime.
2. Remain engine-agnostic and target existing engines.
3. Build an intermediate runtime/ABI while allowing multiple backends.

Explain which approach is best for the MVP and which is best for the long-term vision.

### H. Website/database

Design the data model and architecture for:

* components
* implementations
* interfaces
* games/reference sources
* licenses
* provenance
* dependencies
* versions
* tests
* benchmarks
* users
* repositories
* compatibility

Keep the initial infrastructure cheap and simple.

### I. Security and trust

Consider:

* malicious components
* supply-chain attacks
* arbitrary code execution
* dependency attacks
* untrusted repositories
* sandboxing
* AI-generated malicious code
* license violations

### J. Legal/IP architecture

Do not provide definitive legal advice.

Instead, identify:

* major legal risks
* what the platform should never host
* how provenance should work
* how clean-room implementations could be represented
* how reference games should be represented
* how licenses should be tracked
* what questions require an IP lawyer

### K. Business/ecosystem strategy

Explain how this could become a network-effect platform.

Consider:

* why developers contribute components
* discovery
* ratings
* benchmarks
* compatibility reputation
* component forks
* versioning
* attribution
* commercial components
* open-source components
* AI-generated components

### L. Development plan

Give me three plans:

1. One-evening prototype
2. One-week serious MVP
3. 1–3 month alpha

For each, specify exactly what should be implemented and what should be deliberately postponed.

### M. Most importantly

Be critical.

Do not simply tell me the idea is good.

Identify the fundamental technical obstacles that could make this fail.

In particular, investigate whether a genuinely general-purpose "LEGO for game software" abstraction is feasible, or whether game systems are intrinsically too coupled to make this work beyond carefully designed systems.

If universal composability is unrealistic, propose the narrowest abstraction that could still produce a compelling product. "Compelling" here means: a developer or agent gets real value from the catalog/spec format even if zero automatic code composition ever ships — because if the honest answer is "only valuable once full composition works," that's a sign the scope is wrong, not a sign to oversell the timeline.

I want a technically rigorous architecture, not a startup pitch.
