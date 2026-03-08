# Table of Contents
This is the official user manual (English edition) for **mdbook-plotly** (hereinafter referred to as “this project”). It provides detailed instructions on various usage methods. If you already know what you need, feel free to jump directly to the corresponding section using the table of contents below.

> [!NOTE]
> This user manual is available in multiple languages; however, not all language versions are guaranteed to reflect the latest application updates. In case of discrepancies among different language versions, the Chinese version shall prevail.


- [Getting Started](#getting-started)
- [Configuration](#Configuration)
- [Input Formats](#input-formats)
    - [JSON](#JSON Input Format)
        - [Important Notes for Document Understanding](#important-notes-for-document-understanding)
        - [Types](#Types)
        - [Main Chart Format](#main-chart-format)
    - [SandboxScript](#SandboxScript)
- [Output Formats](#output-formats)

# Getting Started
### Installation
1. Using Cargo
```shell
cargo install mdbook-plotly
# Alternatively, if you use `binstall`:
cargo binstall mdbook-plotly 
```

Alternatively, you may download the latest available [Releases](https://github.com/TickPoints/mdbook-plotly/releases) compatible with your system from the Releases page on GitHub, then add the application’s installation directory to your system’s PATH environment variable.

2. Configure Your Book
Add the following lines to your book’s `book.toml` file:
```toml
[preprocessor.plotly]
after = ["links"]
```

This configuration enables the basic JSON input format. For other supported input formats, refer to [Input Formats](#Input Formats). Note that code blocks labeled either `plot` or `plotly` will both render interactive Plotly charts.

> [!NOTE]
> We use **JSON5**, which supports comments, trailing commas, unquoted object keys, single-quoted strings, hexadecimal numbers, and multi-line strings. While this introduces a slight performance overhead, it significantly improves usability. After careful evaluation, we determined that this trade-off is well worth it.

In this JSON-based input format, you typically only need threee top-level fields: `data`, `layout` and `config`:

~~~markdown
```plot
{
    layout: {
        title: "Test",
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

This example sets the chart title to "Test", renders a pie chart with the values `[10, 20, 30, 40]`, and disables interactivity (i.e., renders a static, non-interactive plot).

# Configuration
Open your book’s `book.toml` file to add the configuration:

```toml
[preprocessor.plotly]
after = ["links"]
```

Before adding custom configurations, please note the following parsing principles:

1. All configuration keys follow `kebab-case` naming.
2. Invalid or unrecognized configuration keys are silently ignored.
3. A configuration key that is syntactically valid but has an incorrect type will cause the entire configuration parsing to fail. In such cases, all settings fall back to their default values, and an error message similar to the one below is displayed:
```shell
Illegal config format for 'preprocessor.mdbook-plotly': unknown variant `plotlyhtml`, expected `plotly-html`       |  in `output-type` 
``` 
4. If no custom configuration is found (which usually occurs when the `[preprocessor.plotly]` section is missing from `book.toml`—since its presence implies configuration exists), a warning is issued:
```shell
Custom config not found; using default configuration.
```
If this warning appears unexpectedly (e.g., the `[preprocessor.plotly]` section is present), please file a bug report (issue) on GitHub.

Below is a complete list of all available configuration options:

```toml
[preprocessor.plotly]
after = ["links"]

# The following values represent the default settings.

# Output format — determines the final rendered output of the code block.
# This string corresponds to an enum (see [Output Formats](#output-formats) for valid variants).
# Any unrecognized value will cause configuration parsing to fail.
output-type = "plotly-html"

# Input format — specifies the syntax and structure expected for data inside the code block.
# This string also corresponds to an enum (see [Input Formats](#input-formats) for valid variants).
# Any unrecognized value will cause configuration parsing to fail.
input-type = "json-input"

# Script source control.
# This expected true or false (default).
offline_js_sources = false
```

# Input Formats
Input formats determine how you define and configure your charts. They are controlled by the `input-type` configuration option, with the following currently supported values:
- `json-input`
- `sandbox-script`

Because input formats involve nuanced syntax and behavior, each is described in detail below.

## JSON Input Format
This format allows you to define and configure charts using JSON.

We implement our own deserialization logic. In most cases, you can use the same JSON structure as that passed to `Plotly.newPlot()` in JavaScript—but compatibility is not guaranteed, and we may extend the schema beyond Plotly’s native specification. For reliable usage, always refer to the documented fields below. If you require a field not currently supported, please open an issue—we’ll consider adding it in a future release.

### Important Notes for Document Understanding
```json5
{
    // Comments like this are used throughout the documentation to clarify intent.
    // (If you’d like support for comments in actual input files, please file an issue.)

    data?: [
        // The `?` symbol indicates that this field is *optional*.
        {
            type: "pie",  // No `?` → this field is *required*.
            // In some cases, the presence of one field (e.g., `type: "pie"`)
            // implies that certain other fields *must* be present.
            // We will explicitly note such dependencies in the documentation.
            values: [usize; usize]  // Required when `type` is `"pie"`.
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

### Types
JSON provides several primitive types; all composite types are built from these fundamentals.

#### 1. Objects
```json5
// Curly braces `{}` denote an object.
{}

// Keys may be quoted or unquoted (JSON5 supports both);
// `:` separates the key from its value, forming a key-value pair.
{
    "example": "wiki",
    // Trailing commas and unquoted keys are permitted in JSON5.
    example2: true,
}

// In the documentation, a `?` after a key name means the field is optional.
// The value following `:` specifies the *expected type*, not a literal value.
{
    "example"?: String
}

// Sometimes the value is a literal (not a type name), meaning the field is restricted to those exact values.
// The `|` operator denotes a union: the field may be *any one* of the listed options.
{
    "example1"?: "a",
    "example2"?: "a" | "b",
    "example": String | bool
}
```

#### 2. Arrays (Lists)
```json5
// Square brackets `[]` denote an array.
[]

// Elements are comma-separated.
[1, 2, 3, 4]

// In the documentation, arrays typically require all elements to be of the *same type*.
// Syntax `[T; N]` means “an array of `N` elements, each of type `T`”.
// The notation for `N` (the length) is explained below.
[String; 6]

// Occasionally, heterogeneous arrays are allowed—each position has a fixed, explicit type.
[String, usize, String | bool, bool]
```

#### 3. Numeric Types
```json5
// An unquoted number (e.g., `1`) is a numeric literal.
1

// In the documentation, numeric types include:
// - `usize`: Non-negative integer (e.g., `0`, `42`).
// - `isize`: Signed integer (e.g., `-1`, `100`).
// - `f64`: 64-bit floating-point number (e.g., `0.1`, `-3.14`).
// All numeric types have theoretical upper/lower bounds; exceeding them may result in undefined behavior—
// e.g., parsing failure or unpredictable numeric values.

// Ranges are expressed as inclusive-exclusive intervals:
// `0..6f64` means “any `f64` value ≥ 0.0 and < 6.0”.
0..6f64

// Unions of ranges (or types) are expressed with `|`:
0..6usize | 10..1000f64
```

#### 4. Strings
```json5
// A sequence of characters enclosed in double quotes (`"..."`) is a string literal.
"str"

// In the documentation, `String` denotes the generic string type.
String

// Enum-like string literals are common and often defined via union (`|`):
"a" | "b" | "c"
```

#### 5. Booleans
```json5
// Boolean literals are unquoted `true` or `false`.
true
false

// In the documentation, `bool` denotes the boolean type.
bool

// Booleans may appear in unions with other types:
String | bool
```

#### Common Complex Types

- **Rgb**

_Definition_:
```json5
// A specially formatted string where the `u8` values are the customizable numeric parts.
// NOTE: If the string format is invalid, parsing will fail.
Rgb: "rgb(u8, u8, u8)"
```
_Example_:
```json5
"rgb(0, 0, 0)"
```

- **Rgba**

_Definition_:
```json5
// A specially formatted string where the `u8` and `f64` values are the customizable numeric parts.
// NOTE: If the string format is invalid, parsing will fail.
Rgba: "rgba(u8, u8, u8, f64)"
```
_Example_:
```json5
"rgba(0, 0, 0, 0.0)"
```

- **Color**
Color is a particularly complex composite type composed of multiple variants.

_Definition_:
```json5
// Exactly one of the following options must be present.
// (Having multiple or none will cause parsing failure.)
Color: {
    // A predefined named color
    named_color?: NamedColor,
    // Construct a valid RGB color from R, G, and B channels
    rgb_color?: Rgb,
    // Construct a valid RGBA color from R, G, B, and A channels
    rgba_color?: Rgba,
}

// Cross-browser compatible predefined colors (CSS definition: https://www.w3schools.com/cssref/css_colors.php)
// NOTE: Uses lowercase naming—e.g., AliceBlue should be written as "aliceblue".
// NOTE: This is a very long enum; not all values are listed here, but an invalid value will cause parsing failure.
NamedColor: String
```

_Example_:
```json5
{ named_color: "aliceblue" }
```
```json5
{ rgba: "rgba(0, 0, 0, 0)" }
```

### Chart Main Format
```json5
{
    // Build a mapping table for populating mappings in the sections below.
    // Currently unstable—its structure may change frequently in upcoming releases.
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
    // Whether to resize the layout when the window is resized
    // WARNING: This option currently has no practical effect.
    responsive?: bool,
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

## SandboxScript
> [!WARNING]
> **This format is deprecated**. Using it will emit a warning and a debug message, and fall back to rendering the default chart.

This format allows you to define a script that runs in a local sandboxed environment. Upon execution, the script generates the corresponding chart object.

# Output Formats
Output formats determine whether the final rendered result is HTML, SVG, or another format. Each format has its own advantages and trade-offs—we leave the choice to the user.

Output format must be configured globally; for details, see [Configuration](#Configuration).

| Raw Name | Formatted Name | Effect | Additional Notes |
|--------|--------|--------|--------|
| **PlotlyHtml** | `plotly-html`  | Outputs an `<div>` element and a companion `<script>` containing Plotly logic | May cause compatibility issues with Markdown parsers that do not support raw HTML; less suitable for client-side rendering scenarios |
| **PlotlySvg**  | `plotly-svg`   | **TODO** | Not yet implemented; intended to perform most rendering locally, but may increase build time |
