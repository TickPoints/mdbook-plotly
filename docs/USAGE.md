# Table of Contents
This is the official user manual (English edition) for **mdbook-plotly** (hereinafter referred to as “this project”). It provides detailed instructions on various usage methods. If you already know what you need, feel free to jump directly to the corresponding section using the table of contents below.

> [!NOTE]
> This user manual is available in multiple languages; however, not all language versions are guaranteed to reflect the latest application updates. In case of discrepancies among different language versions, the Chinese version shall prevail.


- [Getting Started](#getting-started)
- [Configuration](#Configuration)
- [Input Formats](#input-formats)
    - [JSON](#JSON)
        - [Important Notes for Document Understanding](#important-notes-for-document-understanding)
        - [Types](#Types)
        - [Main Chart Format](#main-chart-format)
        - [Layout Format](#layout-format)
        - [Config Format](#config-format)
        - [Data-pie](#Data-pie)
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

#### Generic Complex Types
- Rgb
_Definition_:
```json5
Rgb: "rgb(usize, usize, usize)"
```

_Example_:
```json5
{
    layout: {
        plot_background_color: "rgb(255, 255, 255)"
    }
}
```

> [!NOTE]
>  Rgb values must be three integers in the range 0–255, formatted as "rgb(r, g, b)". Hex (#RRGGBB) or named colors (e.g., "white") are not supported in this format.

### Main Chart Format
```json5
{
    // Chart layout configuration
    layout?: Layout,

    // Chart data series
    data?: [Data; usize],

    // Plot-level configuration options (e.g., interactivity, rendering behavior)
    config?: Configuration
}
```

> [!WARNING]
> The detailed definitions of Layout, Data, and Configuration are still under development. If you’d like to help improve this documentation, please consider submitting a pull request (PR).

### Layout Format
```json5
layout: {
    // Chart title (displayed at the top)
    title?: String,

    // Whether to display the legend
    show_legend?: bool,

    // Chart height (in pixels)
    // NOTE: Charts are responsive by default. Explicitly setting `height` may cause layout issues on certain devices or screen sizes.
    height?: usize,

    // Chart width (in pixels)
    // NOTE: As with `height`, explicit `width` may interfere with responsive behavior. Use with caution.
    width?: usize,

    // Color palette used for sequential or categorical data (e.g., pie slices, bar segments)
    // Colors are applied in order; if more traces exist than colors, the palette cycles.
    colorway?: [Rgb; usize],

    // Background color of the plotting area (i.e., the region inside the axes)
    plot_background_color?: Rgb,

    // Decimal and thousands separators for numeric labels
    separators?: String,

    // Legend configuration
    legend: {
        background_color?: Rgb,
        border_color?: Rgb,
        border_width?: usize,
        x?: f64,           // Horizontal position (0–1, relative to plot width)
        y?: f64,           // Vertical position (0–1, relative to plot height)
        trace_group_gap?: usize,  // Spacing (px) between legend groups
        title?: String     // Legend title (if shown)
    },

    // Margins around the plot area (in pixels)
    margin: {
        left?: usize,      // Left margin
        right?: usize,     // Right margin
        top?: usize,       // Top margin
        bottom?: usize,    // Bottom margin
        pad?: usize,       // Padding between margin and plot content
        auto_expand?: bool // Whether margins automatically expand to fit content (default: `true`)
    }
}
```

### Config Format
```josn5
// No comments have been added to this section.
config: {
    static_plot?: bool,
    typeset_math?: bool,
    editable?: bool,
    autosizable?: bool,
    responsive?: bool,
    fill_frame?: bool,
    frame_margins?: f64,
    scroll_zoom?: bool,
    show_axis_drag_handles?: bool,
    show_axis_range_entry_boxes?: bool,
    show_tips?: bool,
    show_link?: bool,
    send_data?: bool
}
```

### Data-bar
`bar` may be a `Data`. This `Data` will be rendered as a bar chart.
```json5
// No comments have been added to this section.
{
    type: "bar",

    x: [f64; usize],
    y: [f64; usize],

    ids?: [String; usize],
    offset?: f64,
    offset_array?: [f64; usize],
    text?: String,
    text_array?: [String; usize],
    text_template?: String,
    hover_template?: String,
    hover_template_array?: [String; usize],
    hover_text?: String,
    hover_text_array?: [String; usize],
    name?: String,
    opacity?: f64,
    x_axis?: String,
    y_axis?: String,
    alignment_group?: String,
    offset_group?: String,
    clip_on_axis?: bool,
}
```

### Data-pie
`pie` may be a `Data`. This `Data` will be rendered as a pie chart.
```json5
// No comments have been added to this section.
{
    type: "pie",

    values: [f64; usize],

    automargin?: bool,
    dlabel?: f64,
    hole?: f64,
    hover_template?: String,
    hover_template_array?: [String; usize],
    hover_text?: String,
    hover_text_array?: [String; usize],
    ids?: [String; usize],
    label0?: f64,
    labels?: [String; usize],
    legend_group?: String,
    legend_rank?: usize,
    name?: String,
    opacity?: f64,
    meta?: String,
    sort?: bool,
    text_position_src?: String,
    text_position_src_array?: [String; usize],
    text?: String,
    text_array?: [String; usize],
    text_info?: String
}
```

### Data-scatter
`scatter` may be a `Data`. This `Data` will be rendered as a scatter chart.
```json5
// No comments have been added to this section.
{
    type: "scatter",

    x: [f64; usize],
    y: [f64; usize],

    web_gl_mode?: bool,
    x0?: f64,
    dx?: f64,
    y0?: f64,
    dy?: f64,
    ids?: [String; usize],
    text?: String,
    text_array?: [String; usize],
    text_template?: String,
    hover_template?: String,
    hover_template_array?: [String; usize],
    hover_text?: String,
    hover_text_array?: [String; usize],
    name?: String,
    opacity?: f64,
    meta?: String,
    x_axis?: String,
    y_axis?: String,
    stack_group?: String,
    clip_on_axis?: bool,
    connect_gaps?: bool,
    fill_color?: Rgba,
    fill?: "tozeroy" | "tozerox" | "tonexty" | "tonextx" | "toself" | "tonext" | "none",
    mode?: "lines"| "markers" | "text" | "linesmarkers" | "linestext" | "markerstext" | "linemarkerstext" | "none",
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
