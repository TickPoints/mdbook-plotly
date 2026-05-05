# mdbook-plotly 用户手册

这是 **mdbook-plotly** 的官方用户手册（中文版），该预处理器可在 mdbook 文档中渲染交互式或静态的 Plotly 图表。本手册提供全面的参考和使用说明。

> [!NOTE]
> 本用户手册提供多种语言版本；但是，并非所有语言版本都保证反映应用程序的最新更新。如果不同语言版本之间存在差异，应以中文版为准。

## 目录

- [快速开始](#快速开始)
  - [安装](#安装)
  - [基本配置](#基本配置)
  - [第一个图表示例](#第一个图表示例)
- [配置参考](#配置参考)
  - [配置语法](#配置语法)
  - [配置选项](#配置选项)
- [输入格式](#输入格式)
  - [JSON Input](#json-input)
    - [JSON 语法与类型系统](#json-语法与类型系统)
    - [映射与生成器](#映射与生成器)
    - [图表主格式](#图表主格式)
    - [Layout格式](#layout格式)
    - [Config格式](#config格式)
    - 轨迹类型
      - [条形图](#data-bar)
      - [散点图](#data-scatter)
      - [饼图](#data-pie)
      - [直方图](#data-histogram)
      - [蜡烛图](#data-candlestick)
      - [OHLC 图](#data-ohlc)
      - [图像轨迹](#data-image)
      - [桑基图](#data-sankey)
      - [地理散点图](#data-scatter_geo)
      - [Mapbox 散点图](#data-scatter_mapbox)
      - [极坐标散点图](#data-scatter_polar)
      - [Mapbox 密度热力图](#data-density_mapbox)
      - [表格](#data-table)
  - [SandBoxScript（已弃用）](#sand-box-script)
- [输出格式](#输出格式)

## 快速开始

本节将指导您安装 mdbook-plotly、配置您的 mdbook 并创建第一个图表。

### 安装

#### 使用 Cargo

```shell
cargo install mdbook-plotly
```

如果使用 `cargo-binstall`：

```shell
cargo binstall mdbook-plotly
```

#### 手动下载

从 [Releases](https://github.com/TickPoints/mdbook-plotly/releases) 页面下载适用于您平台的最新版本，并将二进制文件添加到系统的 PATH 环境变量中。

### 基本配置

在您的 `book.toml` 文件中添加以下内容：

```toml
[preprocessor.plotly]
after = ["links"]
```

此配置启用默认的 JSON5 输入格式。使用 `plot` 或 `plotly` 语言的代码块将被处理成交互式 Plotly 图表。

> [!NOTE]
> mdbook-plotly 使用 **JSON5** 语法，它在 JSON 基础上扩展了注释、尾随逗号、无引号的对象键、单引号字符串、十六进制数字和多行字符串。这提高了图表定义的可读性和可维护性。

### 第一个图表示例

一个最小的图表定义需要三个顶层字段：`data`、`layout` 和 `config`：

````markdown
```plot
{
    layout: {
        title: "测试图表",
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
````

此示例：

- 将图表标题设置为 "测试图表"
- 创建一个包含四个扇区的饼图
- 禁用交互性（静态图表）

有关可用图表类型、配置选项和高级功能的详细信息，请参阅以下部分。

## 配置参考

mdbook-plotly 的配置选项在 `book.toml` 的 `[preprocessor.plotly]` 部分中指定。

### 配置语法

所有配置键均使用 `kebab-case`。解析器遵循以下规则：

1. **未知键被忽略** – 无法识别的配置键将被静默丢弃。
2. **类型敏感验证** – 键的类型无效（例如，需要布尔值时提供了字符串）将导致整个配置被拒绝。所有设置将恢复为默认值，并记录错误。
3. **缺失部分警告** – 如果缺少 `[preprocessor.plotly]` 部分，将发出警告并使用默认值。如果该部分存在但仍出现警告，请提交错误报告。

提供无效枚举变体时的错误示例：

```shell
Illegal config format for 'preprocessor.mdbook-plotly': unknown variant `plotlyhtml`, expected `plotly-html`       |  in `output-type`
```

### 配置选项

```toml
[preprocessor.plotly]
after = ["links"]

# 输出格式 – 决定渲染的图表格式。
# 有效值："plotly-html"、"plotly-svg"（实验性）
output-type = "plotly-html"

# 输入格式 – 指定图表定义的语法。
# 有效值："json-input"、"sandbox-script"（已弃用）
input-type = "json-input"

# 是否使用离线 JavaScript 源（true/false）。
offline_js_sources = false
```

## 输入格式

`input-type` 配置选项决定在 `plot`/`plotly` 代码块内部定义图表所使用的语法。支持的值：

- `json-input` – 基于 JSON5 的图表定义（推荐）
- `sand-box-script` – 已弃用的基于脚本的格式

### JSON Input

这是主要且推荐的格式。图表使用 JSON5 语法定义，它在标准 JSON 基础上扩展了注释、尾随逗号、无引号键等便利功能。

> [!NOTE]
> mdbook‑plotly 实现了自己的反序列化逻辑。虽然结构通常遵循 Plotly 的原生模式，但不保证完全兼容，并且提供了扩展（例如映射引用和生成器）。请始终参考下面记录的字段以确保可靠使用。缺失的字段可以通过 GitHub issues 请求添加。

#### JSON 语法与类型系统

本参考中使用以下符号来描述预期的类型和可选性。

```json5
{
    // 字段名后的 `?` 表示该字段是可选的。
    data?: [
        {
            // 没有 `?` 表示该字段是必需的。
            type: "pie",
            // 当另一个字段具有特定值时，某些字段变为必需。
            // 此类依赖关系将在文档中注明。
            values: [usize; usize]   // 当 `type` 为 `"pie"` 时必需。
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

##### 基本类型

- **对象** – `{ "key": value }` 或 `{ key: value }`（JSON5 允许无引号键）
- **数组** – `[value1, value2, ...]`；符号 `[T; N]` 表示包含 `N` 个元素、每个元素类型为 `T` 的数组
- **字符串** – `"text"` 或 `'text'`；`String` 表示任意字符串
- **数字** – `usize`（非负整数）、`isize`（有符号整数）、`f64`（浮点数）
- **布尔值** – `true` 或 `false`
- **联合类型** – `"a" | "b"` 表示值可以是 `"a"` 或 `"b"`
- **范围** – `0..6f64` 表示任意 ≥ 0.0 且 < 6.0 的 `f64` 值

##### 通用复合类型

- **Rgb** – `"rgb(u8, u8, u8)"`（例如 `"rgb(0, 0, 0)"`）
- **Rgba** – `"rgba(u8, u8, u8, f64)"`（例如 `"rgba(0, 0, 0, 0.0)"`）
- **Color** – 可以是：
  - 命名的 CSS 颜色：`"aliceblue"`
  - RGB 字符串：`"rgb(255, 0, 0)"`
  - RGBA 字符串：`"rgba(255, 0, 0, 0.5)"`

##### 复杂通用类型

`marker` 是一个嵌套对象，用于控制数据点的视觉样式，包括颜色、透明度、尺寸、符号形状、颜色标尺等。此对象适用于支持 `marker` 字段的 trace 类型（如 `scatter`、`bar`、`scatterpolar` 等）。

```json5
{
    // ===== 基本视觉属性 =====
    // 数据点颜色
    color?: Color,
    // 不透明度，取值范围 0（完全透明）~ 1（完全不透明）
    opacity?: f64,
    // 数据点的大小（像素或与 size_mode 配合的解释单位）
    size?: usize,
    // 每个数据点的单独尺寸
    size_array?: [usize; usize],

    // 数据点符号形状
    symbol?: "circle" | "square" | "diamond" | "cross" | "x" | "triangle-up" | "triangle-down" | "triangle-left" | "triangle-right" | "pentagon" | "hexagon",
    // 尺寸模式：
    //   "area"     – size 值代表标记的面积（默认）
    //   "diameter" – size 值代表标记的直径
    size_mode?: "area" | "diameter",

    // ===== 尺寸与显示控制 =====
    // 显示的最大数据点数量（超出部分将被隐藏）
    max_displayed?: usize,
    // 尺寸参考值，用于自定义尺寸映射
    size_ref?: usize,
    // 最小尺寸限制
    size_min?: usize,

    // ===== 颜色标度（用于颜色编码数值）=====
    // 是否自动计算颜色标度的最小值/最大值
    cauto?: bool,
    // 颜色标度最大值
    cmax?: f64,
    // 颜色标度最小值
    cmin?: f64,
    // 颜色标度中间值（用于发散色条）
    cmid?: f64,
    // 是否自动选择颜色标度
    auto_color_scale?: bool,
    // 是否反转颜色标度
    reverse_scale?: bool,
    // 是否显示颜色条
    show_scale?: bool,
    // 异常值颜色（仅在某些 trace 中有效）
    outlier_color?: Color,
}
```

#### 映射与生成器

`map` 字段提供了一个映射表，可以在图表定义的其他地方通过 `map.key` 语法引用。这允许重复使用数据并通过内置生成器生成复杂值。

映射值可以是原始数据（任何 JSON 值）或生成器对象。生成器对象具有一个 `type` 字段表示生成算法，以及额外的参数。

#### 生成器类型

所有生成器对象必须具有 `type` 字段。支持以下生成器类型：

- **`raw`** — 直接传递数据，不做更改。

## 参数：

  ```json5
  {
      type: "raw",
      data: T  // 任何可直接使用的值
  }
  ```

## 示例：

  ```json5
  { type: "raw", data: [1, 2, 3] }
  ```

- **`g-number-list`** — 通过对一个整数范围内的每个整数求值表达式来生成数字列表。

## 参数：

  ```json5
  {
      type: "g-number-list",
      begin: usize,   // 起始索引（包含）
      end: usize,     // 结束索引（不包含）
      expr: String    // 使用变量 `i` 的算术表达式
  }
  ```

  表达式使用 [fasteval](https://crates.io/crates/fasteval) 库求值；变量 `i`（作为 `f64`）在表达式中可用。

## 示例：

  ```json5
  { type: "g-number-list", begin: 0, end: 3, expr: "i * 2" }
  // 得到 [0.0, 2.0, 4.0]
  ```

- **`g-number`** — 求值一个常数算术表达式。

## 参数：

  ```json5
  {
      type: "g-number",
      expr: String    // 算术表达式（无变量）
  }
  ```

## 示例：

  ```json5
  { type: "g-number", expr: "2 + 3 * 4" }
  // 得到 14.0
  ```

- **`g-range`** — 生成浮点数的算术级数。

## 参数：

  ```json5
  {
      type: "g-range",
      begin: f64,     // 第一个值（包含）
      end: f64,       // 上界（不包含）
      step?: f64      // 步长（默认为 1.0，必须为正数）
  }
  ```

## 示例：

  ```json5
  { type: "g-range", begin: 0.0, end: 5.0, step: 1.0 }
  // 得到 [0.0, 1.0, 2.0, 3.0, 4.0]
  ```

- **`g-repeat`** — 将给定值重复指定次数。

## 参数：

  ```json5
  {
      type: "g-repeat",
      value: T,       // 任意 JSON 值
      count: usize    // 重复次数
  }
  ```

## 示例：

  ```json5
  { type: "g-repeat", value: 42.0, count: 3 }
  // 得到 [42.0, 42.0, 42.0]
  ```

- **`g-linear`** — 在 `begin` 和 `end` 之间（包含两端点）线性生成 `count` 个值。

## 参数：

  ```json5
  {
      type: "g-linear",
      begin: f64,
      end: f64,
      count: usize    // 必须为正数
  }
  ```

  如果 `count` 为 1，结果为 `[begin]`。否则步长为 `(end - begin) / (count - 1)`。

## 示例：

  ```json5
  { type: "g-linear", begin: 0.0, end: 1.0, count: 5 }
  // 得到 [0.0, 0.25, 0.5, 0.75, 1.0]
  ```

- **`g-random`** — 生成随机数。可以生成单个值或数组，支持整数或浮点数，可选种子以确保可复现。

## 参数：

  ```json5
  {
      type: "g-random",
      min: f64,           // 最小值（包含）
      max: f64,           // 最大值（不包含）
      integer?: bool,     // 是否生成整数（默认 false）
      seed?: u64,         // 随机种子（可选，相同种子产生相同序列）
      count?: u64         // 生成数量（省略则生成单个值）
  }
  ```

## 示例：

  ```json5
  // 生成单个 0~100 之间的浮点数
  { type: "g-random", min: 0, max: 100 }

  // 生成 5 个 1~10 之间的整数（固定种子）
  { type: "g-random", min: 1, max: 11, integer: true, seed: 42, count: 5 }
  ```

- **`g-choose`** — 从给定的选项列表中随机选取。可以选取单个值或数组，可选种子。

## 参数：

  ```json5
  {
      type: "g-choose",
      options: [T; usize],  // 候选值列表（不能为空）
      seed?: u64,           // 随机种子（可选）
      count?: u64           // 选取数量（省略则选取单个值）
  }
  ```

## 示例：

  ```json5
  { type: "g-choose", options: ["red", "green", "blue", "yellow"], seed: 1, count: 3 }
  // 可能得到 ["blue", "red", "yellow"]
  ```

- **`g-env`** — 读取环境变量的值。可在构建时注入配置，可选默认值。

## 参数：

  ```json5
  {
      type: "g-env",
      name: String,         // 环境变量名称
      default?: String      // 环境变量不存在时的默认值
  }
  ```

  如果环境变量未设置且没有 `default`，将报错。

## 示例：

  ```json5
  { type: "g-env", name: "MAPBOX_TOKEN", default: "" }
  // 读取 $MAPBOX_TOKEN，不存在则返回 ""
  ```

- **`g-join`** — 将字符串数组用指定的分隔符拼接成单个字符串。

## 参数：

  ```json5
  {
      type: "g-join",
      values: [String; usize],  // 要拼接的字符串数组
      separator?: String        // 分隔符（默认为空字符串）
  }
  ```

## 示例：

  ```json5
  { type: "g-join", values: ["a", "b", "c"], separator: ", " }
  // 得到 "a, b, c"
  ```

- **`if`** — 根据算术表达式的结果在两个值之间进行条件选择。

## 参数：

  ```json5
  {
      type: "if",
      condition: String,    // 算术表达式，求值为数字
      true: T,              // 当 condition ≠ 0.0 时使用的值
      false: T              // 当 condition = 0.0 时使用的值
  }
  ```

  条件使用 [fasteval](https://crates.io/crates/fasteval) 库求值；表达式中没有可用变量。比较运算（如 `2 > 1`）结果为 `1.0`（真）或 `0.0`（假）。

## 示例：

  ```json5
  { type: "if", condition: "2 > 1", true: [1,2,3], false: [4,5,6] }
  // 得到 [1,2,3]，因为 2 > 1 求值为 1.0（非零）
  ```

- **`time`** — 在两个时间点之间按指定的时间间隔生成时间序列。

## 参数：

  ```json5
  {
      type: "time",
      start: String,        // 起始时间字符串
      end: String,          // 结束时间字符串
      interval: String,     // 时间间隔（如 "1d", "2h", "30m"）
      format?: String       // 可选的输出格式（strftime 语法，默认 RFC 3339）
  }
  ```

  支持的时间字符串格式：RFC 3339、`YYYY-MM-DDTHH:MM:SS`、`YYYY-MM-DD HH:MM:SS`、`YYYY-MM-DD`。
  还支持相对时间：`now`、`now+1d`、`now-2h` 等。

  支持的间隔单位：`s`（秒）、`m`（分钟）、`h`（小时）、`d`（天）、`w`（周）。可以组合使用，如 `"1d12h30m"`。

## 示例：

  ```json5
  { type: "time", start: "2026-01-01", end: "2026-01-03", interval: "1d" }
  // 得到 ["2026-01-01T00:00:00+00:00", "2026-01-02T00:00:00+00:00", "2026-01-03T00:00:00+00:00"]

  { type: "time", start: "now", end: "now+3d", interval: "1d", format: "%Y-%m-%d" }
  // 得到 [今天, 明天, 后天, 大后天]，格式为 "YYYY-MM-DD"
  ```

### 使用映射

映射条目在其他地方通过前缀 `map.` 引用。例如，如果映射包含键 `myrange`，您可以在任何接受 `DataPack<T>` 的字段（大多数数组和数字字段）中使用 `"map.myrange"`。

## 完整示例：

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

由于绝大部分都兼容 `DataPack<T>`，所以这里只给出不兼容的内容：

- 任何 `type` 字段（通常是硬编码在代码中的，所以不建议更改）
- Map 中的原始生成器

另外要注意的是，生成器在每一次处理的时候都是懒加载的，并且相同的生成器在不同的字段之间不共享数据，因而每一次使用 Map 都将会重新求值。
如果你发现有的条目没有在上面给出，却不支持Map，请提交一个Issue，我们将会解决。

### 图表主格式

```json5
{
    // 构建映射表，用来在下面的内容中填充映射
    map?: Map,

    // 图表的布局
    layout?: Layout,

    // 图表的数据
    data?: [Data; usize],

    // 图表的配置
    config?: Configuration,
}
```

#### Layout格式

```json5
layout: {
    // 此表的表头(标题)
    title?: String,
    // 是否展示说明
    show_legend?: bool,
    // 图表的高度
    // NOTE: 图表可以自动适应，修改可能导致部分应用设备上大小不良
    height?: usize,
    // 图表的宽度
    // NOTE: 图表可以自动适应，修改可能导致部分应用设备上大小不良
    width?: usize,
    // 图表的色轨
    // 例如饼图中需要多种颜色的饼，程序就会按顺序从色轨中取颜色
    colorway?: [Color; usize],
    // 图表的背景颜色
    plot_background_color?: Color,
    // 分离器
    separators?: String,
    // 是否启用自动尺寸调整（默认 true）
    auto_size?: bool,
    // 图纸背景色（区别于 plot_background_color）
    paper_background_color?: Color,

    // 说明器具体设置
    legend?: {
        // 背景色
        background_color?: Color,
        // 边框色
        border_color?: Color,
        // 边框宽度
        border_width?: usize,
        // X轴长度
        x?: f64,
        // Y轴长度
        y?: f64,
        // 与Data间隙
        trace_group_gap?: usize,
        // 标题
        title?: String,
        // 图例项宽度（像素）
        item_width?: usize,

        // 图例中 trace 的显示顺序
        // "normal": 正常顺序
        // "reversed": 倒序
        // "grouped": 按组显示
        // "reversed+grouped": 倒序分组
        trace_order?: "normal" | "reversed" | "grouped" | "reversed+grouped",

        // 图例项尺寸模式
        // "trace": 按 trace 宽度
        // "constant": 恒定宽度
        item_sizing?: "trace" | "constant",

        // 单击图例项的效果
        // "toggle": 切换该 trace
        // "toggleothers": 切换其他 trace
        // "false": 无操作
        item_click?: "toggle" | "toggleothers" | "false",

        // 双击图例项的效果（取值同 item_click）
        item_double_click?: "toggle" | "toggleothers" | "false",

        // 图例文本垂直对齐
        // "top": 顶对齐 / "middle": 居中 / "bottom": 底对齐
        valign?: "top" | "middle" | "bottom",

        // 单击图例分组的效果
        // "toggleitem": 切换组内所有项
        // "togglegroup": 切换整个组
        group_click?: "toggleitem" | "togglegroup",
    },

    // 图表边缘具体设置
    margin?: {
        // 左边缘宽度
        left?: usize,
        // 右边缘宽度
        right?: usize,
        // 上边缘宽度
        top?: usize,
        // 下边缘宽度
        bottom?: usize,
        // 边缘宽度
        // NOTE: 该选项会覆盖其他边缘宽度设置
        // 不推荐与其他边缘宽度设置一起使用
        pad?: usize,
        // 自动扩张
        auto_expand?: bool
    },
}
```

#### Config格式

```json5
config: {
    // 静态图表
    static_plot?: bool,
    // 数学排版
    // 当页面存在MathJax时有效
    typeset_math?: bool,
    // 决定是否可编辑
    editable?: bool,
    // 决定是否进行自动大小设置
    autosizable?: bool,
    // 决定在窗口大小调整时是否改变布局大小
    // WARNING: 该选项现无实际用处
    responsive?: bool,
    // 决定是否填充屏幕
    // NOTE: 只有当autosizable为true时有效
    fill_frame?: bool,
    // 边缘宽度
    // NOTE: 只有当autosizable为true时有效
    frame_margins?: f64,
    // 确定是否启用鼠标滚轮或双指滚动缩放
    // NOTE: 对于笛卡尔子图默认禁用，其他则默认开启
    scroll_zoom?: bool,
    // 显示笛卡尔坐标轴的平移/缩放拖动手柄
    show_axis_drag_handles?: bool,
    // 显示平移/缩放时的范围输入
    // NOTE: 只有当show_axis_drag_handles为true时有效
    show_axis_range_entry_boxes?: bool,
    // 对于交互图表是否显示提示
    show_tips?: bool,
    // 是否在图表右下角显示指向 Chart Studio Cloud 的链接
    show_link?: bool,
    // 是否包含仅链接到 Chart Studio Cloud 文件的数据
    // NOTE: 只有当show_link为true时有效
    send_data?: bool,
    // 一些双击操作的可用间隔
    double_click_delay?: usize,
    // Mapbox 访问令牌
    mapbox_access_token?: String,
    // 设置撤销/重做队列的长度
    queue_length?: usize,
    // 确定 plotly 标志是否显示在模式栏的末尾
    display_logo?: bool,
    // 使用公司标志对图像进行水印处理
    watermark?: bool,

    // 控制模式栏的显示行为
    // "hover": 仅悬停时显示
    // "true": 始终显示（默认）
    // "false": 不显示
    display_mode_bar?: "hover" | "true" | "false",

    // 双击图表的响应行为
    // "false": 无操作
    // "reset": 重置视图
    // "autosize": 自动调整大小
    // "reset+autosize": 重置并自动调整
    double_click?: "false" | "reset" | "autosize" | "reset+autosize",

    // 是否显示“在 Chart Studio 中编辑”链接（show_link 的别名）
    show_edit_in_chart_studio?: bool,
}
```

#### Data-bar

`bar`可以是一个`Data`。该`Data`将被渲染为条形图。

```json5
{
    type: "bar",

    // x轴坐标数据
    x: [f64; usize],
    // y轴坐标数据
    y: [f64; usize],

    // 各数据点的唯一标识符
    ids?: [String; usize],
    // 所有柱子相对于默认位置的统一偏移量
    offset?: f64,
    // 为每个柱子单独设置偏移量
    offset_array?: [f64; usize],
    // 显示在柱子上的统一文本
    text?: String,
    // 为每个柱子单独设置显示文本
    text_array?: [String; usize],
    // 文本模板，支持变量替换（如 "%{x}", "%{y}"）
    text_template?: String,
    // 悬停标签模板
    hover_template?: String,
    // 为每个数据点单独设置悬停模板
    hover_template_array?: [String; usize],
    // 悬停时显示的统一文本
    hover_text?: String,
    // 为每个数据点单独设置悬停文本
    hover_text_array?: [String; usize],
    // trace 名称，显示在图例和悬停信息中
    name?: String,
    // 不透明度，取值范围 0（完全透明）~ 1（完全不透明）
    opacity?: f64,
    // 指定绑定的 x 轴（用于多轴图表，如 "x2"）
    x_axis?: String,
    // 指定绑定的 y 轴（用于多轴图表，如 "y2"）
    y_axis?: String,
    // 对齐分组标识，同组的柱子会在坐标轴方向上对齐
    alignment_group?: String,
    // 偏移分组标识，同组的柱子共享偏移空间
    offset_group?: String,
    // 是否裁剪超出坐标轴范围的部分
    clip_on_axis?: bool,
    // 是否在图例中显示此 trace
    show_legend?: bool,
    // 图例分组标识，同组的 trace 在图例中归为一组
    legend_group?: String,
    // 柱子的宽度（数据坐标单位）
    width?: f64,
    // 柱子上文本的旋转角度（单位：度）
    text_angle?: f64,
    // 柱子方向："v" 为垂直柱状图，"h" 为水平条形图
    orientation?: "v" | "h",
    // 控制数据点的视觉样式
    marker?: Marker,
}
```

#### Data-candlestick

`candlestick`可以是一个`Data`。该`Data`将被渲染为蜡烛图，金融可视化中最常见的价格走势表现形式。

```json5
{
    type: "candlestick",

    // x 轴数据（通常为日期字符串，如 "2024-01-15"）
    x: [String; usize],
    // 开盘价
    open: [f64; usize],
    // 最高价
    high: [f64; usize],
    // 最低价
    low: [f64; usize],
    // 收盘价（close > open 时为阳线，反之为阴线）
    close: [f64; usize],

    // trace 名称，显示在图例和悬停信息中
    name?: String,
    // 是否在图例中显示此 trace
    show_legend?: bool,
    // 图例分组标识
    legend_group?: String,
    // 不透明度，取值范围 0 ~ 1
    opacity?: f64,
    // 数据点上显示的统一文本
    text?: String,
    // 为每个数据点单独设置显示文本
    text_array?: [String; usize],
    // 悬停时显示的统一文本
    hover_text?: String,
    // 为每个数据点单独设置悬停文本
    hover_text_array?: [String; usize],
    // 影线（whisker）宽度，取值范围 0 ~ 1（0 为无影线横杠，1 为与实体等宽）
    whisker_width?: f64,
    // 指定绑定的 x 轴（用于多轴图表，如 "x2"）
    x_axis?: String,
    // 指定绑定的 y 轴（用于多轴图表，如 "y2"）
    y_axis?: String,
    // 可见性控制
    // "true": 可见（默认）
    // "false": 不可见
    // "legendonly": 不绘制但可在图例中显示
    visible?: "true" | "false" | "legendonly",
}
```

#### Data-density_mapbox

`densitymapbox`可以是一个`Data`。该`Data`将被渲染为地图密度热力图。

> [!NOTE]
> 使用此 `trace` 需要在 `Layout` 中配置对应的 `mapbox` 对象。

```json5
{
    type: "density_mapbox",

    // 各数据点的纬度
    lat: [f64; usize],
    // 各数据点的经度
    lon: [f64; usize],
    // 各数据点的权重值，决定密度强度
    z: [f64; usize],

    // 是否在图例中显示此 trace
    show_legend?: bool,
    // trace 名称，显示在图例和悬停信息中
    name?: String,
    // 图例分组标识，同组的 trace 在图例中归为一组
    legend_group?: String,
    // 图例排序优先级，数值越小越靠前
    legend_rank?: usize,
    // 不透明度，取值范围 0 ~ 1
    opacity?: f64,
    // 每个数据点的影响半径（单位：像素，默认 30）
    radius?: u8,
    // 地图缩放级别
    zoom?: u8,
    // 是否根据数据自动计算 zmin 和 zmax
    zauto?: bool,
    // 颜色映射的下界
    zmin?: f64,
    // 颜色映射的中间值
    zmid?: f64,
    // 颜色映射的上界
    zmax?: f64,
    // 指定使用的 mapbox 子图（如 "mapbox"、"mapbox2"）
    subplot?: String,
}
```

#### Data-histogram

`histogram`可以是一个`Data`。该`Data`将被渲染为直方图。

```json5
{
    type: "histogram",

    // x 轴数据（x 和 y 至少需要提供一个）
    // 仅提供 x 时创建水平方向直方图
    x?: [f64; usize],
    // y 轴数据（x 和 y 至少需要提供一个）
    // 仅提供 y 时创建垂直方向直方图
    // 同时提供 x 和 y 时创建双变量直方图
    y?: [f64; usize],

    // trace 名称，显示在图例和悬停信息中
    name?: String,
    // 是否在图例中显示此 trace
    show_legend?: bool,
    // 图例分组标识
    legend_group?: String,
    // 不透明度，取值范围 0 ~ 1
    opacity?: f64,
    // 柱子上显示的统一文本
    text?: String,
    // 为每个柱子单独设置显示文本
    text_array?: [String; usize],
    // 悬停时显示的统一文本
    hover_text?: String,
    // 为每个柱子单独设置悬停文本
    hover_text_array?: [String; usize],
    // 悬停标签模板
    hover_template?: String,
    // 为每个柱子单独设置悬停模板
    hover_template_array?: [String; usize],
    // 是否自动确定 x 轴的分箱数量
    auto_bin_x?: bool,
    // x 轴的分箱数量（auto_bin_x 为 false 时生效）
    n_bins_x?: usize,
    // 是否自动确定 y 轴的分箱数量
    auto_bin_y?: bool,
    // y 轴的分箱数量（auto_bin_y 为 false 时生效）
    n_bins_y?: usize,
    // 对齐分组标识，同组的柱子在坐标轴方向上对齐
    alignment_group?: String,
    // 偏移分组标识，同组的柱子共享偏移空间
    offset_group?: String,
    // 分箱分组标识，同组的直方图共享分箱边界
    bin_group?: String,
    // 指定绑定的 x 轴（用于多轴图表，如 "x2"）
    x_axis?: String,
    // 指定绑定的 y 轴（用于多轴图表，如 "y2"）
    y_axis?: String,
    // 柱子方向："v" 为垂直，"h" 为水平
    orientation?: "v" | "h",
    // 直方图聚合函数，决定每个分箱内的计算方式
    // "count": 计数（默认）
    // "sum": 求和
    // "avg": 平均值
    // "min": 最小值
    // "max": 最大值
    hist_func?: "count" | "sum" | "avg" | "min" | "max",
    // 直方图归一化方式
    // "": 不归一化（默认）
    // "percent": 以百分比表示
    // "probability": 以概率表示（总和为 1）
    // "density": 概率密度（面积积分为 1）
    // "probability density": 概率密度（与 density 类似）
    hist_norm?: "" | "percent" | "probability" | "density" | "probability density",
    // 控制数据点的视觉样式
    marker?: Marker,
}
```

#### Data-ohlc

`ohlc`可以是一个`Data`。该`Data`将被渲染为 OHLC 图，常用于金融股票走势分析。

```json5
{
    type: "ohlc",

    // x 轴数据（通常为日期字符串，如 "2024-01-15"）
    x: [String; usize],
    // 开盘价
    open: [f64; usize],
    // 最高价
    high: [f64; usize],
    // 最低价
    low: [f64; usize],
    // 收盘价（close > open 时视为上涨，反之为下跌）
    close: [f64; usize],

    // trace 名称，显示在图例和悬停信息中
    name?: String,
    // 是否在图例中显示此 trace
    show_legend?: bool,
    // 图例分组标识
    legend_group?: String,
    // 不透明度，取值范围 0 ~ 1
    opacity?: f64,
    // 悬停时显示的统一文本
    hover_text?: String,
    // 为每个数据点单独设置悬停文本
    hover_text_array?: [String; usize],
    // 顶部和底部横线（tick）的宽度，取值范围 0 ~ 0.5
    tick_width?: f64,
    // 可见性控制
    // "true": 可见（默认）
    // "false": 不可见
    // "legendonly": 不绘制但可在图例中显示
    visible?: "true" | "false" | "legendonly",
}
```

#### Data-image

`image`可以是一个`Data`。该`Data`将被渲染为像素图像，支持在笛卡尔坐标系中通过二维像素数组直接显示图像。

```json5
{
    type: "image",

    // 图像像素数据
    z: [[Rgb; unsize]; unsize],

    // 不透明度，取值范围 0（完全透明）~ 1（完全不透明）
    opacity?: f64,
    // trace 名称，显示在图例和悬停信息中
    name?: String,
    // 图例排序优先级，数值越小越靠前
    legend_rank?: usize,
    // 图像上显示的统一文本
    text?: String,
    // 为每个像素单独设置显示文本
    text_array?: [String; usize],
    // 悬停时显示的统一文本
    hover_text?: String,
    // 为每个像素单独设置悬停文本
    hover_text_array?: [String; usize],
    // 悬停标签模板
    hover_template?: String,
    // 为每个像素单独设置悬停模板
    hover_template_array?: [String; usize],
    // 使用 data URI 指定图像源（如 "data:image/png;base64,..."）
    // 设置后 z 数据将被忽略
    source?: String,
    // 图像左下角的 x 坐标（默认 0）
    x0?: f64,
    // 像素在 x 方向上的间距（默认 1）
    dx?: f64,
    // 图像左下角的 y 坐标（默认 0）
    y0?: f64,
    // 像素在 y 方向上的间距（默认 1）
    dy?: f64,
    // 指定绑定的 x 轴（用于多轴图表，如 "x2"）
    x_axis?: String,
    // 指定绑定的 y 轴（用于多轴图表，如 "y2"）
    y_axis?: String,
    // 各数据点的唯一标识符
    ids?: [String; usize],
    // 元数据，可在模板中通过 %{meta} 引用
    meta?: String,
    // 图像平滑算法
    // "fast": 快速平滑
    // "false": 不平滑（显示原始像素）
    z_smooth?: "fast" | "false",
}
```

#### Data-pie

`pie`可以是一个`Data`。该`Data`将会被渲染为饼图。

```json5
{
    type: "pie",

    // 各扇区的数值大小
    values: [f64; usize],

    // 是否自动调整边距以防止文本被裁剪
    automargin?: bool,
    // 相邻标签之间的增量值（与 label0 配合使用）
    dlabel?: f64,
    // 中心空洞的比例，取值范围 0 ~ 1（0 为完整饼图，>0 为甜甜圈图）
    hole?: f64,
    // 悬停标签模板
    hover_template?: String,
    // 为每个扇区单独设置悬停模板
    hover_template_array?: [String; usize],
    // 悬停时显示的统一文本
    hover_text?: String,
    // 为每个扇区单独设置悬停文本
    hover_text_array?: [String; usize],
    // 各扇区的唯一标识符
    ids?: [String; usize],
    // 起始标签的数值（与 dlabel 配合自动生成标签）
    label0?: f64,
    // 各扇区的标签文本
    labels?: [String; usize],
    // 图例分组标识
    legend_group?: String,
    // 图例排序优先级，数值越小越靠前
    legend_rank?: usize,
    // trace 名称，显示在图例和悬停信息中
    name?: String,
    // 不透明度，取值范围 0 ~ 1
    opacity?: f64,
    // 元数据，可在模板中通过 %{meta} 引用
    meta?: String,
    // 是否按数值大小对扇区排序
    sort?: bool,
    // 文本位置来源标识
    text_position_src?: String,
    // 为每个扇区单独设置文本位置来源标识
    text_position_src_array?: [String; usize],
    // 扇区上显示的统一文本
    text?: String,
    // 为每个扇区单独设置显示文本
    text_array?: [String; usize],
    // 控制显示的信息内容（如 "percent"、"label"、"value" 或其组合）
    text_info?: String,
    // 是否在图例中显示此 trace
    show_legend?: bool,
    // 饼图的起始旋转角度（单位：度，默认从 12 点钟方向开始）
    rotation?: f64,
    // 扇区拉出距离，取值范围 0 ~ 1（用于突出显示某个扇区）
    pull?: f64,
    // 扇区排列方向
    direction?: "clockwise" | "counterclockwise",
    // 控制数据点的视觉样式
    marker?: Marker,
}
```

#### Data-sankey

`sankey`可以是一个`Data`。该`Data`将被渲染为桑基图，用于可视化节点之间的流量关系。

```json5
{
    type: "sankey",

    // 节点配置
    node?: {
        // 节点颜色
        color?: [Color; usize],
        // 节点之间的间距（像素）
        pad?: f64,
        // 节点的厚度（像素）
        thickness?: f64,
    },

    // trace 名称，显示在图例和悬停信息中
    name?: String,
    // 是否可见
    visible?: bool,
    // 值的格式化字符串（d3-format 语法，如 ".3f" 保留 3 位小数）
    value_format?: String,
    // 值的后缀文本（单位，如 "TWh"、"元"）
    value_suffix?: String,
    // 图的方向："v" 为垂直，"h" 为水平
    orientation?: "v" | "h",
    // 节点排列方式
    // "snap": 对齐到网格（默认）
    // "perpendicular": 垂直排列
    // "freeform": 自由布局（可拖拽）
    // "fixed": 固定位置（不可拖拽）
    arrangement?: "snap" | "perpendicular" | "freeform" | "fixed",
}
```

#### Data-scatter_geo

`scatter_geo`可以是一个`Data`。该`Data`将被渲染为地理散点图，在地理坐标系上绘制。

```json5
{
    type: "scatter_geo",

    // 各数据点的纬度
    lat: [f64; usize],
    // 各数据点的经度
    lon: [f64; usize],

    // 各数据点的唯一标识符
    ids?: [String; usize],
    // 是否在图例中显示此 trace
    show_legend?: bool,
    // trace 名称，显示在图例和悬停信息中
    name?: String,
    // 图例分组标识
    legend_group?: String,
    // 图例排序优先级，数值越小越靠前
    legend_rank?: usize,
    // 不透明度，取值范围 0 ~ 1
    opacity?: f64,
    // 数据点上显示的统一文本
    text?: String,
    // 为每个数据点单独设置显示文本
    text_array?: [String; usize],
    // 文本模板，支持变量替换（如 "%{lat}", "%{lon}"）
    text_template?: String,
    // 为每个数据点单独设置文本模板
    text_template_array?: [String; usize],
    // 悬停时显示的统一文本
    hover_text?: String,
    // 为每个数据点单独设置悬停文本
    hover_text_array?: [String; usize],
    // 悬停标签模板
    hover_template?: String,
    // 为每个数据点单独设置悬停模板
    hover_template_array?: [String; usize],
    // 是否连接缺失数据点之间的间隙
    connect_gaps?: bool,
    // 指定使用的 geo 子图（如 "geo"、"geo2"）
    subplot?: String,
    // 控制此 trace 的图层绘制顺序
    below?: String,
    // 绘制模式，决定如何呈现数据点
    // "lines": 折线
    // "markers": 散点标记
    // "text": 仅文本
    // "linesmarkers": 折线 + 标记
    // "linestext": 折线 + 文本
    // "markerstext": 标记 + 文本
    // "linemarkerstext": 折线 + 标记 + 文本
    // "none": 不显示
    mode?: "lines" | "markers" | "text" | "linesmarkers" | "linestext" | "markerstext" | "linemarkerstext" | "none",
    // 控制数据点的视觉样式
    marker?: Marker,
}
```

#### Data-scatter_mapbox

`scatter_mapbox`可以是一个`Data`。该`Data`将被渲染为 Mapbox 散点图。
> [!NOTE]
> 使用此 `trace` 需要在 `Layout` 中配置对应的 `mapbox` 对象。

```json5
{
    type: "scatter_mapbox",

    // 各数据点的纬度
    lat: [f64; usize],
    // 各数据点的经度
    lon: [f64; usize],

    // 各数据点的唯一标识符
    ids?: [String; usize],
    // 选中的数据点索引列表
    selected_points?: [usize; usize],
    // 是否在图例中显示此 trace
    show_legend?: bool,
    // trace 名称，显示在图例和悬停信息中
    name?: String,
    // 图例分组标识
    legend_group?: String,
    // 图例排序优先级，数值越小越靠前
    legend_rank?: usize,
    // 不透明度，取值范围 0 ~ 1
    opacity?: f64,
    // 数据点上显示的统一文本
    text?: String,
    // 为每个数据点单独设置显示文本
    text_array?: [String; usize],
    // 文本模板，支持变量替换（如 "%{lat}", "%{lon}"）
    text_template?: String,
    // 为每个数据点单独设置文本模板
    text_template_array?: [String; usize],
    // 悬停时显示的统一文本
    hover_text?: String,
    // 为每个数据点单独设置悬停文本
    hover_text_array?: [String; usize],
    // 悬停标签模板
    hover_template?: String,
    // 为每个数据点单独设置悬停模板
    hover_template_array?: [String; usize],
    // 指定使用的 mapbox 子图（如 "mapbox"、"mapbox2"）
    subplot?: String,
    // 控制此 trace 的图层绘制顺序
    below?: String,
    // 元数据，可在模板中通过 %{meta} 引用
    meta?: String,
    // 绘制模式，决定如何呈现数据点
    // "lines": 折线
    // "markers": 散点标记
    // "text": 仅文本
    // "linesmarkers": 折线 + 标记
    // "linestext": 折线 + 文本
    // "markerstext": 标记 + 文本
    // "linemarkerstext": 折线 + 标记 + 文本
    // "none": 不显示
    mode?: "lines" | "markers" | "text" | "linesmarkers" | "linestext" | "markerstext" | "linemarkerstext" | "none",
    // 控制数据点的视觉样式
    marker?: Marker,
}
```

#### Data-scatter_polar

`scatter_polar`可以是一个`Data`。该`Data`将被渲染为极坐标散点图。

```json5
{
    type: "scatter_polar",

    // 角度坐标（单位：度，除非布局中指定了其他单位）
    theta: [f64; usize],
    // 径向坐标
    r: [f64; usize],

    // trace 名称，显示在图例和悬停信息中
    name?: String,
    // 是否在图例中显示此 trace
    show_legend?: bool,
    // 图例分组标识
    legend_group?: String,
    // 不透明度，0（透明）~ 1（不透明）
    opacity?: f64,
    // 数据点上显示的统一文本
    text?: String,
    // 为每个数据点单独设置显示文本
    text_array?: [String; usize],
    // 悬停时显示的统一文本
    hover_text?: String,
    // 为每个数据点单独设置悬停文本
    hover_text_array?: [String; usize],
    // 悬停标签模板
    hover_template?: String,
    // 为每个数据点单独设置悬停模板
    hover_template_array?: [String; usize],
    // 指定使用的极坐标子图（如 "polar"、"polar2"）
    subplot?: String,
    // 是否连接缺失数据点之间的间隙
    connect_gaps?: bool,
    // 径向坐标的参考起始值（与 dr 配合，将数组索引映射为实际径向距离）
    r0?: f64,
    // 径向步长
    dr?: f64,
    // 角度坐标的参考起始值（度）
    theta0?: f64,
    // 角度步长（度）
    dtheta?: f64,
    // 填充类型
    // "tozeroy": 填充到 r=0
    // "tozerox": 填充到 theta=0
    // "tonexty": 填充到下一个 trace 的 r 值
    // "tonextx": 填充到下一个 trace 的 theta 值
    // "toself": 填充封闭区域自身
    // "tonext": 填充到下一个 trace
    // "none": 不填充
    fill?: "tozeroy" | "tozerox" | "tonexty" | "tonextx" | "toself" | "tonext" | "none",
    // 绘制模式
    // "lines": 折线
    // "markers": 散点标记
    // "text": 仅文本
    // "linesmarkers": 折线 + 标记
    // "linestext": 折线 + 文本
    // "markerstext": 标记 + 文本
    // "linemarkerstext": 折线 + 标记 + 文本
    // "none": 不显示
    mode?: "lines" | "markers" | "text" | "linesmarkers" | "linestext" | "markerstext" | "linemarkerstext" | "none",
    // 可见性控制
    // "true": 可见（默认）
    // "false": 不可见
    // "legendonly": 不绘制但在图例中显示
    visible?: "true" | "false" | "legendonly",
    // 控制数据点的视觉样式
    marker?: Marker,
}
```

#### Data-scatter

`scatter`可以是一个`Data`。该`Data`将会被渲染为填充区域图。

```json5
{
    type: "scatter",

    // x轴坐标数据
    x: [f64; usize],
    // y轴坐标数据
    y: [f64; usize],

    // 是否启用 WebGL 渲染（数据量较大时可显著提升性能）
    web_gl_mode?: bool,
    // x 轴起始值（与 dx 配合使用，以线性间隔自动生成 x 坐标）
    x0?: f64,
    // x 轴步长
    dx?: f64,
    // y 轴起始值（与 dy 配合使用，以线性间隔自动生成 y 坐标）
    y0?: f64,
    // y 轴步长
    dy?: f64,
    // 各数据点的唯一标识符
    ids?: [String; usize],
    // 数据点上显示的统一文本
    text?: String,
    // 为每个数据点单独设置显示文本
    text_array?: [String; usize],
    // 文本模板，支持变量替换（如 "%{x}", "%{y}"）
    text_template?: String,
    // 悬停标签模板
    hover_template?: String,
    // 为每个数据点单独设置悬停模板
    hover_template_array?: [String; usize],
    // 悬停时显示的统一文本
    hover_text?: String,
    // 为每个数据点单独设置悬停文本
    hover_text_array?: [String; usize],
    // trace 名称，显示在图例和悬停信息中
    name?: String,
    // 不透明度，取值范围 0 ~ 1
    opacity?: f64,
    // 元数据，可在模板中通过 %{meta} 引用
    meta?: String,
    // 指定绑定的 x 轴（用于多轴图表，如 "x2"）
    x_axis?: String,
    // 指定绑定的 y 轴（用于多轴图表，如 "y2"）
    y_axis?: String,
    // 堆叠分组标识，同组的 trace 将被堆叠在一起
    stack_group?: String,
    // 是否裁剪超出坐标轴范围的部分
    clip_on_axis?: bool,
    // 是否连接缺失数据点（NaN / null）之间的间隙
    connect_gaps?: bool,
    // 填充区域的颜色
    fill_color?: Rgba,
    // 是否在图例中显示此 trace
    show_legend?: bool,
    // 图例分组标识，同组的 trace 在图例中归为一组
    legend_group?: String,
    // 填充区域类型
    // "tozeroy": 填充到 y=0
    // "tozerox": 填充到 x=0
    // "tonexty": 填充到下一个 trace 的 y 值
    // "tonextx": 填充到下一个 trace 的 x 值
    // "toself": 填充封闭区域自身
    // "tonext": 填充到下一个 trace
    // "none": 不填充
    fill?: "tozeroy" | "tozerox" | "tonexty" | "tonextx" | "toself" | "tonext" | "none",
    // 绘制模式，决定如何呈现数据点
    // "lines": 折线图
    // "markers": 散点标记
    // "text": 仅文本
    // "linesmarkers": 折线 + 标记
    // "linestext": 折线 + 文本
    // "markerstext": 标记 + 文本
    // "linemarkerstext": 折线 + 标记 + 文本
    // "none": 不显示
    mode?: "lines" | "markers" | "text" | "linesmarkers" | "linestext" | "markerstext" | "linemarkerstext" | "none",
    // 控制数据点的视觉样式
    marker?: Marker,
}
```

#### Data-table

`table`可以是一个`Data`。该`Data`将被渲染为表格。

```json5
{
    type: "table",

    // 表头数据，每个内层数组代表一列的表头值
    header_values: [[String; usize]; usize],
    // 单元格数据，每个内层数组代表一列的数据值
    // 每列的值数量必须与表头一致
    cells_values: [[String; usize]; usize],

    // trace 名称，显示在图例和悬停信息中
    name?: String,
    // 列宽比例（按比例填充可用宽度）
    column_width?: f64,
    // 数据列的渲染顺序（例如 [2, 0, 1] 表示原第 0 列渲染为第 3 列）
    column_order?: [usize; usize],
    // 可见性控制
    // "true": 可见（默认）
    // "false": 不可见
    // "legendonly": 不绘制但可在图例中显示
    visible?: "true" | "false" | "legendonly",
}
```

### Sand Box Script

> [!WARNING]
> **该格式已被弃用**。使用此格式将输出警告和调试信息，并回退到渲染默认图表。

该格式允许您定义一个在本地沙盒环境中运行的脚本。脚本执行后，将生成相应的图表对象。

## 输出格式

输出格式决定了最终的渲染结果是 HTML、SVG 还是其他格式。每种格式都有其优点和权衡——我们将选择权交给用户。

输出格式必须在全局配置中设置；详情请参阅[配置](#配置参考)。

| 原始名称 | 格式化名称 | 效果 | 其他注意事项 |
|--------|--------|--------|--------|
| **PlotlyHtml** | `plotly-html`  | 输出一个 `<div>` 元素和一个包含 Plotly 逻辑的配套 `<script>` | 可能导致与不支持原始 HTML 的 Markdown 解析器的兼容性问题；不太适合客户端渲染场景 |
| **PlotlySvg**  | `plotly-svg`   | **TODO** | 尚未实现；旨在本地执行大部分渲染，但可能会增加构建时间 |
