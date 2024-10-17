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
fs_mod_parser::parse_basic_mod(file : &Path, is_folder : bool) -> <String>
```
