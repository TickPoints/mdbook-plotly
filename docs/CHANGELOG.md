# CHANGELOG

## v0.1.8-alpha
- Adjusted `code_handler`
    - Updated `map`
- Fixed docs
- Updated tests

## v0.1.7
- Adjusted `code_handler`
    - Updated `map`
- Fixed docs
- Updated tests

## v0.1.7-beta.2
- Adjusted `code_handler`
    - Updated `map`
- Fixed docs
- Adjusted `Cargo.toml`
    - Added the `fasteval` crate

## v0.1.7-beta
- Adjusted `code_handler`
    - Adjusted `must_translate`
    - Added `translate_enum`
    - Made it all to support `DataPack`(Map)
- Updated docs

## v0.1.7-alpha
- Adjusted `Cargo.toml`
- The preview version would now provide the symbols.

## v0.1.6
- Adjusted `Cargo.toml`
    - Adjusted `[profile.release]`
- Updated docs

## v0.1.6-beta
- Adjusted interfaces
    - Adjusted Config
    - Adjusted other Datas
    - Added Color enum
- Updated docs
- Updated tests

## v0.1.6-alpha
- Adjusted interfaces
    - Added Data-candlestick
    - Added Data-sankey
    - Added Data-scatter_polar
    - Added Data-table
    - Added Data-ohlc
- Updated docs

## v0.1.5-alpha
- Adjusted interfaces
    - Added Data-density_mapbox
    - Added Data-histogram
    - Added Data-image
    - Added Data-scatter_geo
    - Added Data-scatter_mapbox
    - Adjusted other Datas
- Updated docs
- Updated some tests

## v0.1.4-alpha
> [!WARNING]
> Although `plotly_svg_handler` is updated, it is not available.

- Updated `plotly_svg_handler`
- Adjusted interfaces
    - Add Data-scatter
    - Adjust Data-bar

## v0.1.3-beta
- Added interfaces
    - Added `Data-bar`
    - Used `usize` instead of `u64`
- Adjusted deps
    - Added `anyhow`
    - Added `env_logger` instead of `chlog`
- Used `anyhow` to optimize

## v0.1.3-alpha
- Added interfaces
    - Added to `Config`
        - `editable`
        - `autosizable`
        - `responsive`
        - `scroll_zoom`
        - `fill_frame`
        - `frame_margins`
        - `show_axis_drag_handlers`
        - `show_axis_range_entry_boxes`
        - `show_tips`
        - `show_link`
        - `send_data`
- Updated `docs/USAGE-zh_CN.md`
- Adjusted deps
    - Added `rand`
- Adjusted features
        - Added `dep:rand` to `plotly-backed`

## v0.1.2-alpha
- Optimized parsers performance in `code_handler`
- Fixed parser bugs

## v0.1.1-alpha
> [!WARNING]
> The changelog of `0.1.0-alpha` has been incorporated into this release.

- Added `plotly-html-handler`
- Added interfaces
    - Added `pie` plot
- Updated docs
- Created workflows
