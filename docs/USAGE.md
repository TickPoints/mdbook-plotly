# mdbook-plotly User Manual

This is the official user manual (English edition) for **mdbook-plotly**, a preprocessor that renders interactive or static Plotly charts in mdbook documentation. The manual provides comprehensive reference and usage instructions.

> [!NOTE]
> This user manual is available in multiple languages; however, not all language versions are guaranteed to reflect the latest application updates. In case of discrepancies among different language versions, the Chinese version shall prevail.

## Table of Contents

- [Quick Start](#quick-start)
    - [Installation](#installation)
    - [Basic Configuration](#basic-configuration)
    - [First Chart Example](#first-chart-example)
- [Configuration Reference](#configuration-reference)
    - [Configuration Syntax](#configuration-syntax)
    - [Configuration Options](#configuration-options)
- [Input Formats](#input-formats)
    - [JSON Input](#json-input)
        - [JSON Syntax and Type System](#json-syntax-and-type-system)
        - [Map and Generators](#map-and-generators)
        - [Chart Main Format](#chart-main-format)
        - [Layout Format](#layout-format)
        - [Config Format](#config-format)
        - Trace Types
            - [Bar Charts](#data-bar)
            - [Scatter Plots](#data-scatter)
            - [Pie Charts](#data-pie)
            - [Histograms](#data-histogram)
            - [Candlestick Charts](#data-candlestick)
            - [OHLC Charts](#data-ohlc)
            - [Image Traces](#data-image)
            - [Sankey Diagrams](#data-sankey)
            - [Geographic Scatter Plots](#data-scatter_geo)
            - [Mapbox Scatter Plots](#data-scatter_mapbox)
            - [Polar Scatter Plots](#data-scatter_polar)
            - [Mapbox Density Heatmaps](#data-density_mapbox)
            - [Tables](#data-table)
    - [SandBoxScript (Deprecated)](#sand-box-script)
- [Output Formats](#output-formats)

# Quick Start

This section guides you through installing mdbook-plotly, configuring your mdbook, and creating your first chart.

## Installation

### Using Cargo
```shell
cargo install mdbook-plotly
```

If you use `cargo-binstall`:
```shell
cargo binstall mdbook-plotly
```

### Manual Download
Download the latest release for your platform from the [Releases](https://github.com/TickPoints/mdbook-plotly/releases) page and add the binary to your system's PATH.

## Basic Configuration

Add the following to your book's `book.toml` file:

```toml
[preprocessor.plotly]
after = ["links"]
```

This configuration enables the default JSON5 input format. Code blocks with language `plot` or `plotly` will be processed into interactive Plotly charts.

> [!NOTE]
> mdbook-plotly uses **JSON5** syntax, which extends JSON with comments, trailing commas, unquoted object keys, single‑quoted strings, hexadecimal numbers, and multi‑line strings. This improves readability and maintainability of chart definitions.

## First Chart Example

A minimal chart definition requires three top‑level fields: `data`, `layout`, and `config`:

~~~markdown
```plot
{
    layout: {
        title: "Test Chart",
    },
    data: [{
        type: "pie",
        values: [10, 20, 30, 40],
    }],
    config: {
        static_plot: true,
    }
}
```
~~~

This example:
- Sets the chart title to "Test Chart"
- Creates a pie chart with four slices
- Disables interactivity (static plot)

For detailed information on available chart types, configuration options, and advanced features, refer to the sections below.

# Configuration Reference

Configuration options for mdbook-plotly are specified in the `[preprocessor.plotly]` section of your `book.toml`.

## Configuration Syntax

All configuration keys use `kebab-case`. The parser follows these rules:

1. **Unknown keys are ignored** – unrecognized configuration keys are silently dropped.
2. **Type‑sensitive validation** – a key with an invalid type (e.g., a string where a boolean is expected) causes the entire configuration to be rejected. All settings then revert to their defaults, and an error is logged.
3. **Missing section warning** – if the `[preprocessor.plotly]` section is absent, a warning is issued and default values are used. If the section is present but the warning appears, please file a bug report.

Example error when an invalid enum variant is supplied:

```shell
Illegal config format for 'preprocessor.mdbook-plotly': unknown variant `plotlyhtml`, expected `plotly-html`       |  in `output-type`
```

## Configuration Options

```toml
[preprocessor.plotly]
after = ["links"]

# Output format – determines the rendered chart format.
# Valid values: "plotly-html", "plotly-svg" (experimental)
output-type = "plotly-html"

# Input format – specifies the syntax of chart definitions.
# Valid values: "json-input", "sandbox-script" (deprecated)
input-type = "json-input"

# Whether to use offline JavaScript sources (true/false).
offline_js_sources = false
```

# Input Formats

The `input-type` configuration option determines the syntax used to define charts inside `plot`/`plotly` code blocks. Supported values:

- `json-input` – JSON5‑based chart definitions (recommended)
- `sand-box-script` – deprecated script‑based format

## JSON Input

This is the primary and recommended format. Charts are defined using JSON5 syntax, which extends standard JSON with comments, trailing commas, unquoted keys, and other conveniences.

> [!NOTE]
> mdbook‑plotly implements its own deserialization logic. While the structure generally follows Plotly’s native schema, compatibility is not guaranteed, and extensions (such as map references and generators) are available. Always refer to the documented fields below for reliable usage. Missing fields can be requested via GitHub issues.

### JSON Syntax and Type System

The following notation is used throughout this reference to describe expected types and optionality.

```json5
{
    // A `?` after a field name indicates the field is optional.
    data?: [
        {
            // No `?` means the field is required.
            type: "pie",
            // Some fields become required when another field has a specific value.
            // Such dependencies are noted in the documentation.
            values: [usize; usize]   // Required when `type` is `"pie"`.
        }
    ],

    layout?: {
        legend?: {
            title?: String,
            background_color?: Rgba
        }
    }
}
```

#### Basic Types

- **Objects** – `{ "key": value }` or `{ key: value }` (JSON5 allows unquoted keys)
- **Arrays** – `[value1, value2, ...]`; notation `[T; N]` means an array of `N` elements each of type `T`
- **Strings** – `"text"` or `'text'`; `String` denotes any string
- **Numbers** – `usize` (non‑negative integer), `isize` (signed integer), `f64` (floating‑point)
- **Booleans** – `true` or `false`
- **Unions** – `"a" | "b"` means the value can be either `"a"` or `"b"`
- **Ranges** – `0..6f64` means any `f64` value ≥ 0.0 and < 6.0

#### Common Composite Types

- **Rgb** – `"rgb(u8, u8, u8)"` (e.g., `"rgb(0, 0, 0)"`)
- **Rgba** – `"rgba(u8, u8, u8, f64)"` (e.g., `"rgba(0, 0, 0, 0.0)"`)
- **Color** – can be one of the following:
  - A named CSS color: `"aliceblue"`
  - An RGB color: `"rgb(255, 0, 0)"`
  - An RGBA color: `"rgba(255, 0, 0, 0.5)"`

### Map and Generators

The `map` field provides a mapping table that can be referenced elsewhere in the chart definition using the `map.key` syntax. This allows reuse of data and generation of complex values via built-in generators.

Map values can be either raw data (any JSON value) or generator objects. Generator objects have a `type` field indicating the generation algorithm, plus additional parameters.

#### Generator Types

The `map` field provides a mapping table that can be referenced elsewhere in the chart definition using the `map.key` syntax. This allows reuse of data and generation of complex values via built-in generators.

Map values can be either raw data (any JSON value) or generator objects. Generator objects have a `type` field indicating the generation algorithm, plus additional parameters.

### Generator Types

All generator objects must have a `type` field. The following generator types are supported:

- **`raw`** — Passes through data unchanged.

  _Parameters:_
  ```json5
  {
      type: "raw",
      data: T  // Any value to be used directly
  }
  ```
  _Example:_
  ```json5
  { type: "raw", data: [1, 2, 3] }
  ```

- **`g-number-list`** — Generates a list of numbers by evaluating an expression for each integer in a range.

  _Parameters:_
  ```json5
  {
      type: "g-number-list",
      begin: usize,   // inclusive start index
      end: usize,     // exclusive end index
      expr: String    // arithmetic expression in variable `i`
  }
  ```
  The expression is evaluated using the [fasteval](https://crates.io/crates/fasteval) library; the variable `i` (as `f64`) is available inside the expression.

  _Example:_
  ```json5
  { type: "g-number-list", begin: 0, end: 3, expr: "i * 2" }
  // yields [0.0, 2.0, 4.0]
  ```

- **`g-number`** — Evaluates a constant arithmetic expression.

  _Parameters:_
  ```json5
  {
      type: "g-number",
      expr: String    // arithmetic expression (no variables)
  }
  ```
  _Example:_
  ```json5
  { type: "g-number", expr: "2 + 3 * 4" }
  // yields 14.0
  ```

- **`g-range`** — Generates an arithmetic progression of floating‑point numbers.

  _Parameters:_
  ```json5
  {
      type: "g-range",
      begin: f64,     // first value (inclusive)
      end: f64,       // upper bound (exclusive)
      step?: f64      // step size (default 1.0, must be positive)
  }
  ```
  _Example:_
  ```json5
  { type: "g-range", begin: 0.0, end: 5.0, step: 1.0 }
  // yields [0.0, 1.0, 2.0, 3.0, 4.0]
  ```

- **`g-repeat`** — Repeats a given value a specified number of times.

  _Parameters:_
  ```json5
  {
      type: "g-repeat",
      value: T,       // any JSON value
      count: usize    // number of repetitions
  }
  ```
  _Example:_
  ```json5
  { type: "g-repeat", value: 42.0, count: 3 }
  // yields [42.0, 42.0, 42.0]
  ```

- **`g-linear`** — Generates `count` values linearly spaced between `begin` and `end` (inclusive of both endpoints).

  _Parameters:_
  ```json5
  {
      type: "g-linear",
      begin: f64,
      end: f64,
      count: usize    // must be positive
  }
  ```
  If `count` is 1, the result is `[begin]`. Otherwise the step is `(end - begin) / (count - 1)`.

  _Example:_
  ```json5
  { type: "g-linear", begin: 0.0, end: 1.0, count: 5 }
  // yields [0.0, 0.25, 0.5, 0.75, 1.0]
  ```

- **`g-random`** — Generates random numbers. Can produce a single value or an array; supports integers or floats; an optional seed ensures reproducibility.

  _Parameters:_
  ```json5
  {
      type: "g-random",
      min: f64,           // minimum value (inclusive)
      max: f64,           // maximum value (exclusive)
      integer?: bool,     // generate integers (default false)
      seed?: u64,         // random seed (optional; same seed → same sequence)
      count?: u64         // number of values to generate (omit for a single value)
  }
  ```
  _Example:_
  ```json5
  // Generate a single floating‑point number between 0 and 100
  { type: "g-random", min: 0, max: 100 }

  // Generate 5 integers between 1 and 10 (fixed seed)
  { type: "g-random", min: 1, max: 11, integer: true, seed: 42, count: 5 }
  ```

- **`g-choose`** — Randomly picks from a list of options. Can pick a single value or an array; optional seed.

  _Parameters:_
  ```json5
  {
      type: "g-choose",
      options: [T; usize],  // candidates (must not be empty)
      seed?: u64,           // random seed (optional)
      count?: u64           // number of picks (omit for a single pick)
  }
  ```
  _Example:_
  ```json5
  { type: "g-choose", options: ["red", "green", "blue", "yellow"], seed: 1, count: 3 }
  // might yield ["blue", "red", "yellow"]
  ```

- **`g-env`** — Reads the value of an environment variable. Useful for injecting build‑time configuration; an optional default can be provided.

  _Parameters:_
  ```json5
  {
      type: "g-env",
      name: String,         // environment variable name
      default?: String      // fallback when the variable is not set
  }
  ```
  If the variable is not set and no `default` is given, an error is raised.

  _Example:_
  ```json5
  { type: "g-env", name: "MAPBOX_TOKEN", default: "" }
  // reads $MAPBOX_TOKEN, returns "" if not set
  ```

- **`g-join`** — Joins an array of strings into a single string using a separator.

  _Parameters:_
  ```json5
  {
      type: "g-join",
      values: [String; usize],  // strings to join
      separator?: String        // separator (default: empty string)
  }
  ```
  _Example:_
  ```json5
  { type: "g-join", values: ["a", "b", "c"], separator: ", " }
  // yields "a, b, c"
  ```

- **`if`** — Conditionally selects between two values based on an arithmetic expression.

  _Parameters:_
  ```json5
  {
      type: "if",
      condition: String,    // arithmetic expression that evaluates to a number
      true: T,              // value used when condition ≠ 0.0
      false: T              // value used when condition = 0.0
  }
  ```
  The condition is evaluated using the [fasteval](https://crates.io/crates/fasteval) library; no variables are available. Comparison operators (e.g., `2 > 1`) yield `1.0` (true) or `0.0` (false).

  _Example:_
  ```json5
  { type: "if", condition: "2 > 1", true: [1,2,3], false: [4,5,6] }
  // yields [1,2,3] because 2 > 1 evaluates to 1.0 (non‑zero)
  ```

- **`time`** — Generates a sequence of timestamps between two time points at a specified interval.

  _Parameters:_
  ```json5
  {
      type: "time",
      start: String,        // start time string
      end: String,          // end time string
      interval: String,     // interval (e.g., "1d", "2h", "30m")
      format?: String       // optional output format (strftime syntax, default RFC 3339)
  }
  ```
  Supported time string formats: RFC 3339, `YYYY-MM-DDTHH:MM:SS`, `YYYY-MM-DD HH:MM:SS`, `YYYY-MM-DD`.
  Relative times are also supported: `now`, `now+1d`, `now-2h`, etc.

  Supported interval units: `s` (seconds), `m` (minutes), `h` (hours), `d` (days), `w` (weeks). Units can be combined, e.g., `"1d12h30m"`.

  _Example:_
  ```json5
  { type: "time", start: "2026-01-01", end: "2026-01-03", interval: "1d" }
  // yields ["2026-01-01T00:00:00+00:00", "2026-01-02T00:00:00+00:00", "2026-01-03T00:00:00+00:00"]

  { type: "time", start: "now", end: "now+3d", interval: "1d", format: "%Y-%m-%d" }
  // yields [today, tomorrow, day after tomorrow, 3 days from now] formatted as "YYYY-MM-DD"
  ```

#### Using the Map

Map entries are referenced elsewhere by prefixing the key with `map.`. For example, if the map contains a key `myrange`, you can refer to it as `"map.myrange"` in any field that accepts a `DataPack<T>` (most array and numeric fields).

_Complete example:_

```json5
{
    map: {
        xs: { type: "g-linear", begin: 0, end: 10, count: 5 },
        ys: { type: "g-number-list", begin: 0, end: 5, expr: "i * i" }
    },
    data: [{
        type: "scatter",
        x: "map.xs",
        y: "map.ys"
    }]
}
```

Since most of them are compatible with `DataPack<T>`, only the incompatible ones are given here:

- Any `type` field (usually hard-coded in the code, so it is not recommended to change)
- Primitive generators in Map

Also note that the generator is lazily loaded each time it is processed, and the same generator does not share data between different fields, so it will be re-evaluated each time a Map is used.

If you find that some items are not given above, but do not support Map, please submit an Issue, and we will solve it.

### Chart Main Format

```json5
{
    // Build a mapping table for populating mappings in the sections below.
    map?: Map,

    // Chart layout configuration
    layout?: Layout,

    // Chart data
    data?: [Data; usize],

    // Chart configuration
    config?: Configuration,
}
```

### Layout Format

```json5
layout: {
    // The chart title
    title?: String,
    // Whether to display the legend
    show_legend?: bool,
    // Chart height
    // NOTE: The chart can auto-resize; modifying this may cause layout issues on some devices.
    height?: usize,
    // Chart width
    // NOTE: The chart can auto-resize; modifying this may cause layout issues on some devices.
    width?: usize,
    // Chart colorway
    // For example, a pie chart requires multiple colors for its slices;
    // the program picks colors sequentially from this colorway.
    colorway?: [Color; usize],
    // Chart background color
    plot_background_color?: Color,
    // Separators
    separators?: String,
    // Whether to enable automatic size adjustment (default true)
    auto_size?: bool,
    // Paper background colour (distinct from plot_background_color)
    paper_background_color?: Color,

    // Legend configuration
    legend?: {
        // Background color
        background_color?: Color,
        // Border color
        border_color?: Color,
        // Border width
        border_width?: usize,
        // X-axis extent
        x?: f64,
        // Y-axis extent
        y?: f64,
        // Gap between trace groups
        trace_group_gap?: usize,
        // Title
        title?: String,
        // Width of each legend item (pixels)
        item_width?: usize,

        // Trace display order within the legend
        // "normal": natural order
        // "reversed": reversed order
        // "grouped": grouped by trace group
        // "reversed+grouped": reversed and grouped
        trace_order?: "normal" | "reversed" | "grouped" | "reversed+grouped",

        // Legend item sizing mode
        // "trace": sized per trace
        // "constant": constant width
        item_sizing?: "trace" | "constant",

        // Single-click behaviour on legend items
        // "toggle": toggle the trace
        // "toggleothers": toggle all other traces
        // "false": no action
        item_click?: "toggle" | "toggleothers" | "false",

        // Double-click behaviour on legend items (same values as item_click)
        item_double_click?: "toggle" | "toggleothers" | "false",

        // Vertical alignment of legend text
        // "top", "middle", "bottom"
        valign?: "top" | "middle" | "bottom",

        // Click behaviour on legend groups
        // "toggleitem": toggle all items in the group
        // "togglegroup": toggle the group
        group_click?: "toggleitem" | "togglegroup",
    },

    // Chart margin configuration
    margin?: {
        // Left margin width
        left?: usize,
        // Right margin width
        right?: usize,
        // Top margin width
        top?: usize,
        // Bottom margin width
        bottom?: usize,
        // Uniform margin width
        // NOTE: This option overrides all individual margin settings.
        // Not recommended to use together with individual margin settings.
        pad?: usize,
        // Auto-expand
        auto_expand?: bool
    },
}
```

### Config Format

```json5
config: {
    // Static chart (disables interactivity)
    static_plot?: bool,
    // Math typesetting
    // Effective when MathJax is present on the page
    typeset_math?: bool,
    // Whether the chart is editable
    editable?: bool,
    // Whether auto-sizing is enabled
    autosizable?: bool,
    // Whether to fill the screen
    // NOTE: Only effective when `autosizable` is `true`.
    fill_frame?: bool,
    // Margin width
    // NOTE: Only effective when `autosizable` is `true`.
    frame_margins?: f64,
    // Whether mouse scroll wheel or two-finger pinch zoom is enabled
    // NOTE: Disabled by default for Cartesian subplots; enabled by default for others.
    scroll_zoom?: bool,
    // Show drag handles for panning/zooming on Cartesian axes
    show_axis_drag_handles?: bool,
    // Show range input boxes when panning/zooming
    // NOTE: Only effective when `show_axis_drag_handles` is `true`.
    show_axis_range_entry_boxes?: bool,
    // Whether to show tips for interactive charts
    show_tips?: bool,
    // Whether to display a link to Chart Studio Cloud in the bottom-right corner of the chart
    show_link?: bool,
    // Whether to include data linked only to Chart Studio Cloud files
    // NOTE: Only effective when `show_link` is `true`.
    send_data?: bool,
    // Available delay interval for certain double-click actions
    double_click_delay?: usize,
    // Mapbox access token
    mapbox_access_token?: String,
    // Set the length of the undo/redo queue
    queue_length?: usize,
    // Whether to display the Plotly logo at the end of the mode bar
    display_logo?: bool,
    // Watermark the image with a company logo
    watermark?: bool,

    // Controls when the modebar appears
    // "hover": only show on hover
    // "true": always show (default)
    // "false": never show
    display_mode_bar?: "hover" | "true" | "false",

    // Double-click behaviour
    // "false": no action
    // "reset": reset the chart view
    // "autosize": autosize
    // "reset+autosize": reset and autosize
    double_click?: "false" | "reset" | "autosize" | "reset+autosize",

    // Whether to show an "Edit in Chart Studio" link (alias for show_link)
    show_edit_in_chart_studio?: bool,
}
```

### Data-bar

`bar` can be a `Data` entry. This `Data` will be rendered as a bar chart.
```json5
{
    type: "bar",

    // X-axis coordinate data
    x: [f64; usize],
    // Y-axis coordinate data
    y: [f64; usize],

    // Unique identifier for each data point
    ids?: [String; usize],
    // Uniform offset for all bars relative to their default positions
    offset?: f64,
    // Individual offset for each bar
    offset_array?: [f64; usize],
    // Uniform text displayed on the bars
    text?: String,
    // Individual text for each bar
    text_array?: [String; usize],
    // Text template with variable substitution (e.g., "%{x}", "%{y}")
    text_template?: String,
    // Hover label template
    hover_template?: String,
    // Individual hover template for each data point
    hover_template_array?: [String; usize],
    // Uniform text displayed on hover
    hover_text?: String,
    // Individual hover text for each data point
    hover_text_array?: [String; usize],
    // Trace name, displayed in the legend and hover info
    name?: String,
    // Opacity, ranging from 0 (fully transparent) to 1 (fully opaque)
    opacity?: f64,
    // Binds this trace to a specific x-axis (for multi-axis charts, e.g., "x2")
    x_axis?: String,
    // Binds this trace to a specific y-axis (for multi-axis charts, e.g., "y2")
    y_axis?: String,
    // Alignment group identifier; bars in the same group are aligned along the axis
    alignment_group?: String,
    // Offset group identifier; bars in the same group share offset space
    offset_group?: String,
    // Whether to clip content that extends beyond the axis range
    clip_on_axis?: bool,
    // Whether to show this trace in the legend
    show_legend?: bool,
    // Legend group identifier; traces in the same group are grouped together in the legend
    legend_group?: String,
    // Bar width (in data coordinate units)
    width?: f64,
    // Rotation angle for text on the bars (in degrees)
    text_angle?: f64,
    // Bar orientation: "v" for vertical bar chart, "h" for horizontal bar chart
    orientation?: "v" | "h",
}
```

### Data-candlestick

`candlestick` can be a `Data` entry. This `Data` will be rendered as a candlestick chart—the most common representation of price movements in financial visualization.
```json5
{
    type: "candlestick",

    // X-axis data (typically date strings, e.g., "2024-01-15")
    x: [String; usize],
    // Opening price
    open: [f64; usize],
    // Highest price
    high: [f64; usize],
    // Lowest price
    low: [f64; usize],
    // Closing price (close > open indicates a bullish candle; otherwise bearish)
    close: [f64; usize],

    // Trace name, displayed in the legend and hover info
    name?: String,
    // Whether to show this trace in the legend
    show_legend?: bool,
    // Legend group identifier
    legend_group?: String,
    // Opacity, ranging from 0 to 1
    opacity?: f64,
    // Uniform text displayed on data points
    text?: String,
    // Individual text for each data point
    text_array?: [String; usize],
    // Uniform text displayed on hover
    hover_text?: String,
    // Individual hover text for each data point
    hover_text_array?: [String; usize],
    // Whisker width, ranging from 0 to 1 (0 = no whisker tick, 1 = same width as the body)
    whisker_width?: f64,
    // Binds this trace to a specific x-axis (for multi-axis charts, e.g., "x2")
    x_axis?: String,
    // Binds this trace to a specific y-axis (for multi-axis charts, e.g., "y2")
    y_axis?: String,
    // Visibility control
    // "true": visible (default)
    // "false": hidden
    // "legendonly": not drawn but shown in the legend
    visible?: "true" | "false" | "legendonly",
}
```

### Data-density_mapbox

`densitymapbox` can be a `Data` entry. This `Data` will be rendered as a density heatmap on a map.

> [!NOTE]
> Using this `trace` requires configuring the corresponding `mapbox` object in `Layout`.

```json5
{
    type: "density_mapbox",

    // Latitude of each data point
    lat: [f64; usize],
    // Longitude of each data point
    lon: [f64; usize],
    // Weight value for each data point, determining density intensity
    z: [f64; usize],

    // Whether to show this trace in the legend
    show_legend?: bool,
    // Trace name, displayed in the legend and hover info
    name?: String,
    // Legend group identifier; traces in the same group are grouped together in the legend
    legend_group?: String,
    // Legend sort priority; lower values appear first
    legend_rank?: usize,
    // Opacity, ranging from 0 to 1
    opacity?: f64,
    // Influence radius for each data point (in pixels, default 30)
    radius?: u8,
    // Map zoom level
    zoom?: u8,
    // Whether to automatically compute zmin and zmax from the data
    zauto?: bool,
    // Lower bound for color mapping
    zmin?: f64,
    // Midpoint for color mapping
    zmid?: f64,
    // Upper bound for color mapping
    zmax?: f64,
    // Specifies the mapbox subplot to use (e.g., "mapbox", "mapbox2")
    subplot?: String,
}
```

### Data-histogram

`histogram` can be a `Data` entry. This `Data` will be rendered as a histogram.
```json5
{
    type: "histogram",

    // X-axis data (at least one of x and y must be provided)
    // Providing only x creates a histogram along the horizontal axis
    x?: [f64; usize],
    // Y-axis data (at least one of x and y must be provided)
    // Providing only y creates a histogram along the vertical axis
    // Providing both x and y creates a bivariate histogram
    y?: [f64; usize],

    // Trace name, displayed in the legend and hover info
    name?: String,
    // Whether to show this trace in the legend
    show_legend?: bool,
    // Legend group identifier
    legend_group?: String,
    // Opacity, ranging from 0 to 1
    opacity?: f64,
    // Uniform text displayed on the bars
    text?: String,
    // Individual text for each bar
    text_array?: [String; usize],
    // Uniform text displayed on hover
    hover_text?: String,
    // Individual hover text for each bar
    hover_text_array?: [String; usize],
    // Hover label template
    hover_template?: String,
    // Individual hover template for each bar
    hover_template_array?: [String; usize],
    // Whether to automatically determine the number of bins for the x-axis
    auto_bin_x?: bool,
    // Number of bins for the x-axis (effective when auto_bin_x is false)
    n_bins_x?: usize,
    // Whether to automatically determine the number of bins for the y-axis
    auto_bin_y?: bool,
    // Number of bins for the y-axis (effective when auto_bin_y is false)
    n_bins_y?: usize,
    // Alignment group identifier; bars in the same group are aligned along the axis
    alignment_group?: String,
    // Offset group identifier; bars in the same group share offset space
    offset_group?: String,
    // Bin group identifier; histograms in the same group share bin boundaries
    bin_group?: String,
    // Binds this trace to a specific x-axis (for multi-axis charts, e.g., "x2")
    x_axis?: String,
    // Binds this trace to a specific y-axis (for multi-axis charts, e.g., "y2")
    y_axis?: String,
    // Bar orientation: "v" for vertical, "h" for horizontal
    orientation?: "v" | "h",
    // Histogram aggregation function, determining how values within each bin are computed
    // "count": count (default)
    // "sum": sum
    // "avg": average
    // "min": minimum
    // "max": maximum
    hist_func?: "count" | "sum" | "avg" | "min" | "max",
    // Histogram normalization mode
    // "": no normalization (default)
    // "percent": expressed as a percentage
    // "probability": expressed as probability (sums to 1)
    // "density": probability density (area integrates to 1)
    // "probability density": probability density (similar to density)
    hist_norm?: "" | "percent" | "probability" | "density" | "probability density",
}
```

### Data-ohlc

`ohlc` can be a `Data` entry. This `Data` will be rendered as an OHLC chart, commonly used for financial stock trend analysis.
```json5
{
    type: "ohlc",

    // X-axis data (typically date strings, e.g., "2024-01-15")
    x: [String; usize],
    // Opening price
    open: [f64; usize],
    // Highest price
    high: [f64; usize],
    // Lowest price
    low: [f64; usize],
    // Closing price (close > open indicates an uptrend; otherwise a downtrend)
    close: [f64; usize],

    // Trace name, displayed in the legend and hover info
    name?: String,
    // Whether to show this trace in the legend
    show_legend?: bool,
    // Legend group identifier
    legend_group?: String,
    // Opacity, ranging from 0 to 1
    opacity?: f64,
    // Uniform text displayed on hover
    hover_text?: String,
    // Individual hover text for each data point
    hover_text_array?: [String; usize],
    // Width of the top and bottom tick marks, ranging from 0 to 0.5
    tick_width?: f64,
    // Visibility control
    // "true": visible (default)
    // "false": hidden
    // "legendonly": not drawn but shown in the legend
    visible?: "true" | "false" | "legendonly",
}
```

### Data-image

`image` can be a `Data` entry. This `Data` will be rendered as a pixel image, supporting direct image display via a 2D pixel array within a Cartesian coordinate system.
```json5
{
    type: "image",

    // Image pixel data
    z: [[Rgb; unsize]; unsize],

    // Opacity, ranging from 0 (fully transparent) to 1 (fully opaque)
    opacity?: f64,
    // Trace name, displayed in the legend and hover info
    name?: String,
    // Legend sort priority; lower values appear first
    legend_rank?: usize,
    // Uniform text displayed on the image
    text?: String,
    // Individual text for each pixel
    text_array?: [String; usize],
    // Uniform text displayed on hover
    hover_text?: String,
    // Individual hover text for each pixel
    hover_text_array?: [String; usize],
    // Hover label template
    hover_template?: String,
    // Individual hover template for each pixel
    hover_template_array?: [String; usize],
    // Image source specified via data URI (e.g., "data:image/png;base64,...")
    // When set, z data will be ignored
    source?: String,
    // X coordinate of the image's bottom-left corner (default 0)
    x0?: f64,
    // Pixel spacing in the x direction (default 1)
    dx?: f64,
    // Y coordinate of the image's bottom-left corner (default 0)
    y0?: f64,
    // Pixel spacing in the y direction (default 1)
    dy?: f64,
    // Binds this trace to a specific x-axis (for multi-axis charts, e.g., "x2")
    x_axis?: String,
    // Binds this trace to a specific y-axis (for multi-axis charts, e.g., "y2")
    y_axis?: String,
    // Unique identifier for each data point
    ids?: [String; usize],
    // Metadata, accessible in templates via %{meta}
    meta?: String,
    // Image smoothing algorithm
    // "fast": fast smoothing
    // "false": no smoothing (shows raw pixels)
    z_smooth?: "fast" | "false",
}
```

### Data-pie

`pie` can be a `Data` entry. This `Data` will be rendered as a pie chart.
```json5
{
    type: "pie",

    // Numeric value for each sector
    values: [f64; usize],

    // Whether to auto-adjust margins to prevent text clipping
    automargin?: bool,
    // Delta value between adjacent labels (used with label0)
    dlabel?: f64,
    // Proportion of the center hole, ranging from 0 to 1 (0 = full pie, >0 = donut chart)
    hole?: f64,
    // Hover label template
    hover_template?: String,
    // Individual hover template for each sector
    hover_template_array?: [String; usize],
    // Uniform text displayed on hover
    hover_text?: String,
    // Individual hover text for each sector
    hover_text_array?: [String; usize],
    // Unique identifier for each sector
    ids?: [String; usize],
    // Starting label value (used with dlabel to auto-generate labels)
    label0?: f64,
    // Label text for each sector
    labels?: [String; usize],
    // Legend group identifier
    legend_group?: String,
    // Legend sort priority; lower values appear first
    legend_rank?: usize,
    // Trace name, displayed in the legend and hover info
    name?: String,
    // Opacity, ranging from 0 to 1
    opacity?: f64,
    // Metadata, accessible in templates via %{meta}
    meta?: String,
    // Whether to sort sectors by value
    sort?: bool,
    // Text position source identifier
    text_position_src?: String,
    // Individual text position source identifier for each sector
    text_position_src_array?: [String; usize],
    // Uniform text displayed on sectors
    text?: String,
    // Individual text for each sector
    text_array?: [String; usize],
    // Controls the displayed information content (e.g., "percent", "label", "value", or combinations)
    text_info?: String,
    // Whether to show this trace in the legend
    show_legend?: bool,
    // Starting rotation angle for the pie chart (in degrees; default starts from the 12 o'clock position)
    rotation?: f64,
    // Pull distance for sectors, ranging from 0 to 1 (used to highlight a specific sector)
    pull?: f64,
    // Sector arrangement direction
    direction?: "clockwise" | "counterclockwise",
}
```

### Data-sankey

`sankey` can be a `Data` entry. This `Data` will be rendered as a Sankey diagram, used for visualizing flow relationships between nodes.
```json5
{
    type: "sankey",

    // Node configuration
    node?: {
        // Node colors
        color?: [Color; usize],
        // Padding between nodes (in pixels)
        pad?: f64,
        // Node thickness (in pixels)
        thickness?: f64,
    },

    // Trace name, displayed in the legend and hover info
    name?: String,
    // Whether the trace is visible
    visible?: bool,
    // Value format string (d3-format syntax, e.g., ".3f" for 3 decimal places)
    value_format?: String,
    // Value suffix text (unit, e.g., "TWh", "$")
    value_suffix?: String,
    // Diagram orientation: "v" for vertical, "h" for horizontal
    orientation?: "v" | "h",
    // Node arrangement mode
    // "snap": snap to grid (default)
    // "perpendicular": perpendicular arrangement
    // "freeform": free layout (draggable)
    // "fixed": fixed position (not draggable)
    arrangement?: "snap" | "perpendicular" | "freeform" | "fixed",
}
```

### Data-scatter_geo

`scatter_geo` can be a `Data` entry. This `Data` will be rendered as a geographic scatter plot, drawn on a geographic coordinate system.

```json5
{
    type: "scatter_geo",

    // Latitude of each data point
    lat: [f64; usize],
    // Longitude of each data point
    lon: [f64; usize],

    // Unique identifier for each data point
    ids?: [String; usize],
    // Whether to show this trace in the legend
    show_legend?: bool,
    // Trace name, displayed in the legend and hover info
    name?: String,
    // Legend group identifier
    legend_group?: String,
    // Legend sort priority; lower values appear first
    legend_rank?: usize,
    // Opacity, ranging from 0 to 1
    opacity?: f64,
    // Uniform text displayed on data points
    text?: String,
    // Individual text for each data point
    text_array?: [String; usize],
    // Text template with variable substitution (e.g., "%{lat}", "%{lon}")
    text_template?: String,
    // Individual text template for each data point
    text_template_array?: [String; usize],
    // Uniform text displayed on hover
    hover_text?: String,
    // Individual hover text for each data point
    hover_text_array?: [String; usize],
    // Hover label template
    hover_template?: String,
    // Individual hover template for each data point
    hover_template_array?: [String; usize],
    // Whether to connect gaps between missing data points
    connect_gaps?: bool,
    // Specifies the geo subplot to use (e.g., "geo", "geo2")
    subplot?: String,
    // Controls the layer drawing order for this trace
    below?: String,
    // Drawing mode, determining how data points are rendered
    // "lines": lines
    // "markers": scatter markers
    // "text": text only
    // "linesmarkers": lines + markers
    // "linestext": lines + text
    // "markerstext": markers + text
    // "linemarkerstext": lines + markers + text
    // "none": hidden
    mode?: "lines" | "markers" | "text" | "linesmarkers" | "linestext" | "markerstext" | "linemarkerstext" | "none",
}
```

### Data-scatter_mapbox

`scatter_mapbox` can be a `Data` entry. This `Data` will be rendered as a Mapbox scatter plot.
> [!NOTE]
> Using this `trace` requires configuring the corresponding `mapbox` object in `Layout`.

```json5
{
    type: "scatter_mapbox",

    // Latitude of each data point
    lat: [f64; usize],
    // Longitude of each data point
    lon: [f64; usize],

    // Unique identifier for each data point
    ids?: [String; usize],
    // List of selected data point indices
    selected_points?: [usize; usize],
    // Whether to show this trace in the legend
    show_legend?: bool,
    // Trace name, displayed in the legend and hover info
    name?: String,
    // Legend group identifier
    legend_group?: String,
    // Legend sort priority; lower values appear first
    legend_rank?: usize,
    // Opacity, ranging from 0 to 1
    opacity?: f64,
    // Uniform text displayed on data points
    text?: String,
    // Individual text for each data point
    text_array?: [String; usize],
    // Text template with variable substitution (e.g., "%{lat}", "%{lon}")
    text_template?: String,
    // Individual text template for each data point
    text_template_array?: [String; usize],
    // Uniform text displayed on hover
    hover_text?: String,
    // Individual hover text for each data point
    hover_text_array?: [String; usize],
    // Hover label template
    hover_template?: String,
    // Individual hover template for each data point
    hover_template_array?: [String; usize],
    // Specifies the mapbox subplot to use (e.g., "mapbox", "mapbox2")
    subplot?: String,
    // Controls the layer drawing order for this trace
    below?: String,
    // Metadata, accessible in templates via %{meta}
    meta?: String,
    // Drawing mode, determining how data points are rendered
    // "lines": lines
    // "markers": scatter markers
    // "text": text only
    // "linesmarkers": lines + markers
    // "linestext": lines + text
    // "markerstext": markers + text
    // "linemarkerstext": lines + markers + text
    // "none": hidden
    mode?: "lines" | "markers" | "text" | "linesmarkers" | "linestext" | "markerstext" | "linemarkerstext" | "none",
}
```

### Data-scatter_polar

`scatter_polar` can be a `Data` entry. This `Data` will be rendered as a scatter plot in polar coordinates.

```json5
{
    type: "scatter_polar",

    // Angular coordinates (degrees unless layout.polar.angularaxis.thetaunit is overridden)
    theta: [f64; usize],
    // Radial coordinates
    r: [f64; usize],

    // Trace name, displayed in legend and hover info
    name?: String,
    // Whether to show this trace in the legend
    show_legend?: bool,
    // Legend group identifier
    legend_group?: String,
    // Opacity, from 0 (transparent) to 1 (opaque)
    opacity?: f64,
    // Uniform text displayed on data points
    text?: String,
    // Per-point text
    text_array?: [String; usize],
    // Uniform hover text
    hover_text?: String,
    // Per-point hover text
    hover_text_array?: [String; usize],
    // Hover label template
    hover_template?: String,
    // Per-point hover template
    hover_template_array?: [String; usize],
    // Polar subplot to use (e.g. "polar", "polar2")
    subplot?: String,
    // Whether to connect gaps between missing data points
    connect_gaps?: bool,
    // Reference start value for radial coordinates (used to map array indices to radial distance)
    r0?: f64,
    // Radial step (used with r0)
    dr?: f64,
    // Reference start value for angular coordinates (degrees)
    theta0?: f64,
    // Angular step (degrees)
    dtheta?: f64,
    // Fill style
    // "tozeroy": fill to r=0
    // "tozerox": fill to theta=0
    // "tonexty": fill to next trace's r values
    // "tonextx": fill to next trace's theta values
    // "toself": fill enclosed area
    // "tonext": fill to next trace
    // "none": no fill
    fill?: "tozeroy" | "tozerox" | "tonexty" | "tonextx" | "toself" | "tonext" | "none",
    // Drawing mode
    // "lines": lines only
    // "markers": markers only
    // "text": text only
    // "linesmarkers": lines + markers
    // "linestext": lines + text
    // "markerstext": markers + text
    // "linemarkerstext": lines + markers + text
    // "none": hidden
    mode?: "lines" | "markers" | "text" | "linesmarkers" | "linestext" | "markerstext" | "linemarkerstext" | "none",
    // Visibility control
    // "true": visible (default)
    // "false": hidden
    // "legendonly": not drawn but shown in legend
    visible?: "true" | "false" | "legendonly",
}
```

### Data-scatter

`scatter` can be a `Data` entry. This `Data` will be rendered as a filled area chart.
```json5
{
    type: "scatter",

    // X-axis coordinate data
    x: [f64; usize],
    // Y-axis coordinate data
    y: [f64; usize],

    // Whether to enable WebGL rendering (can significantly improve performance with large datasets)
    web_gl_mode?: bool,
    // X-axis starting value (used with dx to auto-generate x coordinates at linear intervals)
    x0?: f64,
    // X-axis step size
    dx?: f64,
    // Y-axis starting value (used with dy to auto-generate y coordinates at linear intervals)
    y0?: f64,
    // Y-axis step size
    dy?: f64,
    // Unique identifier for each data point
    ids?: [String; usize],
    // Uniform text displayed on data points
    text?: String,
    // Individual text for each data point
    text_array?: [String; usize],
    // Text template with variable substitution (e.g., "%{x}", "%{y}")
    text_template?: String,
    // Hover label template
    hover_template?: String,
    // Individual hover template for each data point
    hover_template_array?: [String; usize],
    // Uniform text displayed on hover
    hover_text?: String,
    // Individual hover text for each data point
    hover_text_array?: [String; usize],
    // Trace name, displayed in the legend and hover info
    name?: String,
    // Opacity, ranging from 0 to 1
    opacity?: f64,
    // Metadata, accessible in templates via %{meta}
    meta?: String,
    // Binds this trace to a specific x-axis (for multi-axis charts, e.g., "x2")
    x_axis?: String,
    // Binds this trace to a specific y-axis (for multi-axis charts, e.g., "y2")
    y_axis?: String,
    // Stack group identifier; traces in the same group will be stacked together
    stack_group?: String,
    // Whether to clip content that extends beyond the axis range
    clip_on_axis?: bool,
    // Whether to connect gaps between missing data points (NaN / null)
    connect_gaps?: bool,
    // Fill area color
    fill_color?: Rgba,
    // Whether to show this trace in the legend
    show_legend?: bool,
    // Legend group identifier; traces in the same group are grouped together in the legend
    legend_group?: String,
    // Fill area type
    // "tozeroy": fill to y=0
    // "tozerox": fill to x=0
    // "tonexty": fill to the next trace's y values
    // "tonextx": fill to the next trace's x values
    // "toself": fill the enclosed area itself
    // "tonext": fill to the next trace
    // "none": no fill
    fill?: "tozeroy" | "tozerox" | "tonexty" | "tonextx" | "toself" | "tonext" | "none",
    // Drawing mode, determining how data points are rendered
    // "lines": line chart
    // "markers": scatter markers
    // "text": text only
    // "linesmarkers": lines + markers
    // "linestext": lines + text
    // "markerstext": markers + text
    // "linemarkerstext": lines + markers + text
    // "none": hidden
    mode?: "lines" | "markers" | "text" | "linesmarkers" | "linestext" | "markerstext" | "linemarkerstext" | "none",
}
```

### Data-table

`table` can be a `Data` entry. This `Data` will be rendered as a table.
```json5
{
    type: "table",

    // Header data; each inner array represents the header values for one column
    header_values: [[String; usize]; usize],
    // Cell data; each inner array represents the data values for one column
    // The number of values in each column must match the header
    cells_values: [[String; usize]; usize],

    // Trace name, displayed in the legend and hover info
    name?: String,
    // Column width ratio (proportionally fills available width)
    column_width?: f64,
    // Rendering order of data columns (e.g., [2, 0, 1] means the original column 0 is rendered as the 3rd column)
    column_order?: [usize; usize],
    // Visibility control
    // "true": visible (default)
    // "false": hidden
    // "legendonly": not drawn but shown in the legend
    visible?: "true" | "false" | "legendonly",
}
```

## Sand Box Script

> [!WARNING]
> **This format is deprecated**. Using it will emit a warning and a debug message, and fall back to rendering the default chart.

This format allows you to define a script that runs in a local sandboxed environment. Upon execution, the script generates the corresponding chart object.

# Output Formats
Output formats determine whether the final rendered result is HTML, SVG, or another format. Each format has its own advantages and trade-offs—we leave the choice to the user.

Output format must be configured globally; for details, see [Configuration](#configuration-reference).

| Raw Name | Formatted Name | Effect | Additional Notes |
|--------|--------|--------|--------|
| **PlotlyHtml** | `plotly-html`  | Outputs an `<div>` element and a companion `<script>` containing Plotly logic | May cause compatibility issues with Markdown parsers that do not support raw HTML; less suitable for client-side rendering scenarios |
| **PlotlySvg**  | `plotly-svg`   | **TODO** | Not yet implemented; intended to perform most rendering locally, but may increase build time |
