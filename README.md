<div align="center">
    <h1>Tarmac</h1>
</div>

<div align="center">
    <a href="https://github.com/rojo-rbx/tarmac/actions">
        <img src="https://github.com/rojo-rbx/tarmac/workflows/CI/badge.svg" alt="GitHub Actions status" />
    </a>
    <a href="https://crates.io/crates/tarmac">
        <img src="https://img.shields.io/crates/v/tarmac.svg?label=latest%20release" alt="Latest release" />
    </a>
</div>

<hr />

Tarmac is a tool that manages assets for Roblox projects on the command line. It paves the way for hermetic place builds when used with tools like [Rojo](https://github.com/rojo-rbx/rojo).

Tarmac is inspired by projects like [Webpack](https://webpack.js.org/) that allow you to import assets as if they're code.

## Installation

### From GitHub Releases
Pre-built binaries are available for 64-bit Windows, macOS, and Linux from the [GitHub releases page](https://github.com/rojo-rbx/tarmac/releases).

### From Crates.io
Tarmac requires Rust 1.39+ to build.

```bash
cargo install tarmac
```

## Usage
**Check out the [examples](examples) folder for small, working projects using Tarmac.**

To get started, make a `tarmac.toml` file in the root of your project. This is where you can configure where Tarmac will look for assets and what it will do with them.

If you want Tarmac to manage PNG files in a folder named `assets`, you can use:

```toml
name = "01-basic-game"

# Most projects will define some 'inputs'.
# This tells Tarmac where to find assets that we'll use in our game.
[[inputs]]

# For a syntax overview of Tarmac's globs, see:
# https://docs.rs/globset/0.4.4/globset/#syntax
glob = "assets/**/*.png"

# Tells Tarmac what kind of link files to generate.
# This option creates `ModuleScript` instances that return asset URLs.
codegen = "asset-url"
```

Run `tarmac sync --target roblox` to have Tarmac upload any new or updated assets that your project depends on. You may need to pass a `.ROBLOSECURITY` cookie explicitly via the `--auth` argument.

Tarmac will generate Lua code next to each asset that looks something like this:

```lua
-- This file was @generated by Tarmac. It is not intended for manual editing.
return "rbxassetid://4597138422"
```

These files will be turned into `ModuleScript` instances by tools like Rojo and can be loaded as if you're importing the image itself:

```lua
local ImageA = require(Assets.A)

local decal = Instance.new("Decal")
decal.Texture = ImageA
```

For more options, run `tarmac --help`.

## License
Tarmac is available under the MIT license. See [LICENSE.txt](LICENSE.txt) for details.