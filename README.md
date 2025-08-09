# README.md

This is a web-based interactive fiction engine written in Rust, using Actix-Web for backend routing and session management, and Tera for dynamic HTML templating. The focus is on creating a world of interconnected locations in a dynamic, session-aware, and extensible way.

## Goals

- **Node-Based World:** Each location (page) is a node in a graph. Connections between pages create the "map".
- **Session-Based Progress:** User progress and state are stored on the backend using Actix session cookies.
- **Single-Route Navigation:** The URL never changes. Navigation is managed entirely via session data and POST requests.
- **Dynamic Environment:** Pages get contextual information such as season, weather, or events, computed and cached for a short duration.

## Current State

- Basic project skeleton (`main.rs`, `pages.rs`, `session.rs`) implemented.
- Core types: `Page`, `PageGraph`, and `UserSession`.
- Navigation works via POST requests; user's session determines which page to serve.
- Environment context is stubbed and supports cached random values.
- Example locations and transitions set up in code.
- Simple templates display pages and choices as buttons.

## Next Steps

- Add file-based loading for pages (JSON/TOML/YAML).
- Expand environment model: more dynamic weather/events, NPCs, etc.
- Implement player inventory, flags, or richer game logic.
- Add more nuanced actions per location, with custom templates/logic.
- Harden error handling for invalid moves or corrupted state.
- GUI/admin tools for editing the world?