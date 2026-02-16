# CHANGELOG

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
