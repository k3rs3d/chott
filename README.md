# README.md

This is a web-based interactive fiction engine written in Rust, using `actix-web` for routing and session management, and `tera` for dynamic HTML templating. The goal is to create a navigable "world" of interconnected locations which vary by time and randomly. 

Currently in an extremely early state. 

## TODO:

- Add file-based loading for pages (JSON/TOML/YAML)
- Expand environment model: more dynamic weather/events, NPCs, etc
- NPC spawning, interactions, layered routines  
- Event mechanic to notify players of events between page reloads 
- Add more nuanced actions per location, with custom templates/logic
- Harden error handling for invalid moves or corrupted state
- Implement player inventory, flags, or richer game logic?
- GUI/admin tools for editing the world?
