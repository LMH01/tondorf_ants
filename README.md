# Tondorf_ants

This is a client implementation for this game: https://github.com/Tondorf/AntServer

## Todo

- [ ] Reorganize Project (create submodules)
- [ ] Fix crash when ants are fighting and ant dies
- [ ] Make own ants hunt enemy ants when they are below a certain life threshold (also determine if it is possible to "catch" this ant by only attacking when the distance to the ant is lower then the distance from the ant to the own homebase)
- [ ] Make two types of own ants: Resource gatherer (will only gather sugar) and offensive ants. Offensive ants should attack enemies and deliver toxic wast to enemy bases. The ratio should be configurable by constants.
- [ ] Own ants should retreat back to home base when below a certain life threshold
- [ ] Improve pathfinding to homebase by avoiding walking into enemy homebase