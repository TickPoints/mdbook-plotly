# CHANGELOG

## v0.2.2-alpha
- Refined layout parser release-prep cleanup
    - Normalized axis `type` error messages in [`parse_axis_obj()`](src/code_handler/plot_obj_parser/layout_parser.rs:256) to match the field-oriented style already used by [`translate_with_config!()`](src/macros.rs:36) and [`translate_enum_with_config!()`](src/macros.rs:66)
    - Kept axis parsing on explicit [`ParseContext`](src/code_handler/parse_context.rs:4) / [`MapEvalConfig`](src/preprocessor/config.rs:61) propagation without reintroducing hidden defaults
    - Added an axis regression test in [`tests/test_code_handler.rs`](tests/test_code_handler.rs) covering `map.*`-backed `range`, `tick_prefix`, `tick_suffix`, `title`, `anchor`, `overlaying`, `show_tick_labels`, `auto_margin`, `fixed_range`, and axis `type`
- Updated release documentation
    - Clarified that parser migration work favors [`translate_with_config!()`](src/macros.rs:36) and [`translate_enum_with_config!()`](src/macros.rs:66) over legacy [`translate!()`](src/macros.rs:29) in active parsing paths
    - Documented `map-eval` / `map-parser-extensions` behavior more explicitly in user-facing docs
- Adjusted configs
    - Removed `offline_js_sources`

## v0.2.1
- Refined parser migration toward explicit [`MapEvalConfig`](src/preprocessor/config.rs:61) propagation
    - Replaced top-level legacy [`translate!()`](src/macros.rs:29) usage in [`parse_config_obj()`](src/code_handler/plot_obj_parser/layout_parser.rs:8) and [`parse_layout_obj()`](src/code_handler/plot_obj_parser/layout_parser.rs:59) with explicit [`translate_with_config!()`](src/macros.rs:36) entrypoints
- Updated tests to validate the migration path remains stable
    - Added a layout/config regression case in [`tests/test_code_handler.rs`](tests/test_code_handler.rs) covering `map.*`-backed title, legend, axis range, and config parsing

## v0.2.1-beta
- Refined parser migration toward explicit [`MapEvalConfig`](src/preprocessor/config.rs:61) propagation
    - Migrated trace parsers to consume [`ParseContext`](src/code_handler/parse_context.rs:4) instead of raw `map` where applicable
    - Removed [`must_translate()`](src/code_handler/until.rs:29) and migrated remaining translation paths to explicit [`ParseContext`](src/code_handler/parse_context.rs:4) / config-driven helpers
    - Threaded active map-eval config through generator parsing paths in [`src/code_handler/until.rs`](src/code_handler/until.rs)
    - Reused shared helpers like [`parse_marker()`](src/code_handler/plot_obj_parser/common.rs:7) and [`parse_color_bar()`](src/code_handler/plot_obj_parser/common.rs:59) to reduce duplicated translation logic
- Slimmed trace registry dispatch in [`trace_registry.rs`](src/code_handler/plot_obj_parser/trace_registry.rs:1)
    - Removed registry-side `.map(into_trace)` adaptation
    - Moved trait-object conversion boundaries down into parser modules via `*_trace()` entrypoints
- Updated tests to validate the migration path remains stable

## v0.2.1-alpha
- Refined plot parser structure
- Refined parser config propagation
- Updated deps

## v0.2.0
- Added **Layout field expansion** — Phase 1 & 2
    - **Phase 1 (Basic fields)**:
        - Spacing: `bar_gap`, `bar_group_gap`, `box_gap`, `box_group_gap`
        - Interaction: `hover_mode`, `drag_mode`, `click_mode`
        - Sub-objects: `font` (family/size/color), `coloraxis` (cmin/cmax/cmid/auto/reverse/show)
    - **Phase 2 (Axis support)**:
        - Default axes: `xaxis`, `yaxis` with full field support
        - Named axes: `xaxis2`–`xaxis8`, `yaxis2`–`yaxis8`
        - Axis fields: `title`, `show_grid`, `show_line`, `zero_line`, `visible`,
          `anchor`, `overlaying`, `range`, `color`, `line_color`, `grid_color`,
          `tick_prefix`, `tick_suffix`, `tick_format`, `hover_format`, `category_array`,
          `fixed_range`, `scale_anchor`, `auto_margin`, `show_tick_labels`
        - Axis enums: `category_order` (16 variants), `type` (linear/log/date/category/multicategory)
- Added helper functions: `parse_axis_obj()`, `parse_named_axes()`
- Added 3 axis tests (xaxis, date type, named axes)
- Updated documentation (USAGE-zh_CN.md, USAGE.md) with new Layout fields and Axis reference

## v0.2.0-beta
- Added 6 new Data trace types
    - Added `box` (BoxPlot)
    - Added `contour` (Contour)
    - Added `heatmap` (HeatMap)
    - Added `mesh3d` (Mesh3D)
    - Added `scatter3d` (Scatter3D)
    - Added `surface` (Surface)
- Updated docs for all new trace types (USAGE-zh_CN.md, USAGE.md)
- Added comprehensive tests for all existing and new trace types
- Re-exported all new trace types in `src/code_handler/plot_obj_parser.rs`

## v0.1.9
- Updated interfaces
    - Added Marker in `bar`, `histogra`, `pie`, `scatter`, `scatter_geo`, `scatter_mapbox`, `scatter_polar`
- Updated tests and macros

## v0.1.9-beta
- Updated interfaces
    - Updated layout
    - Updated config
    - Updated legend
    - Updated the `Color`
- Fixed docs

## v0.1.9-alpha
- Updated Cargo.toml
    - Adjusted deps
            - Added `chrono`
    - Adjusted features
        - Add the `map-parser-extensions` feature
- Updated interfaces
    - Updated the map parser
            - Updated `time`, `g-random`, `g-choose`
- Fixed docs

## v0.1.8
> [!NOTE]
> Hotfix. Please don't use old versions.
- Fixed `BookData`

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
