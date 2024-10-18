# FS Mod Parser

A rust utility library to read and parse Farming Simulator Mod files

## Parsers Available

### Basic Mod Details

Checks the file name, and returns:

- file metadata
- mod pedigree (author, title, etc..)
- map information (if applicable)
- mod content tests

```rust
let path_to_mod = std::path::Path::new("FS22_Mod_File.zip");
let json_representation = fs_mod_parser::parse_basic_mod(path_to_mod, false);
```
