# Tondorf_ants

This is a client implementation for this game: https://github.com/Tondorf/AntServer

## Todo

- [X] Reorganize Project (create submodules)
- [X] Fix crash when ants are fighting and ant dies
- [X] Make own ants hunt enemy ants when they are below a certain life threshold 
- [X] Make two types of own ants: Resource gatherer (will only gather sugar) and offensive ants. Offensive ants should attack enemies and deliver toxic wast to enemy bases. The ratio should be configurable by constants. Add config option that enables resource gatherers to also hunt low life enemies.
    - [X] The toxic waste part is not yet implemented
- [X] Own ants should retreat back to home base when below a certain life threshold
- [X] Improve pathfinding to homebase by avoiding walking into enemy homebase
- [X] Maybe make config values get provided by command line arguments instead of hard coded constants
- [ ] See if ants deliver toxins deliberatly to own base
- [X] Implement setting of ant jobs by console arguments (backend)
- [ ] Add -r flag that randomly selects ant jobs
