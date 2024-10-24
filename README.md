# FS Mod Parser

A rust utility library to read and parse Farming Simulator Mod files

## Parsers Available

### Basic Mod Details

Checks the file name, and returns:

- file metadata
- mod pedigree (author, title, etc..)
- map information (if applicable)
- mod content tests

Valid input is a file or folder, any type that coerces into a `&Path`.

```rust
let json_representation = fs_mod_parser::parse_mod("FS22_Mod_File.zip").to_json_pretty();
```

### Save Game Details

Returned information includes:

- Mods loaded and used in the save with total count
- Playtime, Save Date, Save Name
- Map mod name and title
- Errors, if any, and boolean valid flag
- Farm list, boolean if it's a multiplayer save or not

Valid input is a file or folder, any type that coerces into a `&Path`.

```rust
let json_representation = fs_mod_parser::parse_savegame("savegame1.zip").to_json_pretty();
```

### Store Item Details

Returned information includes:

- Added Brands
- Added L10N Strings
- Vehicles
- Placables and Productions

Valid input is a file or folder, any type that coerces into a `&Path`.

```rust
let json_representation = fs_mod_parser::parse_detail("FS22_Mod_File.zip").to_json_pretty();
```
