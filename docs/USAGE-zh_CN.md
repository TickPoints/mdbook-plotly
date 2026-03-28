# 目录
这是有关 **mdbook-plotly**(以下简称"本项目") 的具体用户手册(中文版)。它详细介绍了有关本项目的各种使用方法，如果已经知道您需要什么，请直接从目录跳转到对应板块。

> [!NOTE]
> 该用户手册有多个语言的版本，同时并不保证每个语言的版本都符合最新应用程序的情况。当不同语言版本间有冲突时，请以中文版为准。

- [开始使用](#开始使用)
- [配置](#配置)
- [输入格式](#输入格式)
    - [JSON](#JSON输入格式)
        - [文档理解须知](#文档理解须知)
        - [类型](#类型)
        - [映射与生成器](#映射与生成器)
        - [图表主格式](#图表主格式)
    - [SandboxScript](#SandboxScript输入格式)
- [输出格式](#输出格式)

# 开始使用

### 安装
1. 使用Cargo
```shell
cargo install mdbook-plotly
# 如果您使用binstall
cargo binstall mdbook-plotly
```

或者也可以从GitHub页面的[Releases](https://github.com/TickPoints/mdbook-plotly/releases)根据您的系统信息下载最新可用版本，然后把此应用程序所在路径添加到系统的 `Path` 环境变量下。

2. 请在您书的 `book.toml` 添加如下内容:
```toml
[preprocessor.plotly]
after = ["links"]
```
这是基本的输入形式(JSON)，其他输入格式可以参考[输入格式](#输入格式)。另外用`plot`和`plotly`两个名字的代码块均可以生成图表。

> [!NOTE]
> 我们使用了`json5`，它允许注释、尾随逗号、无引号的对象键、单引号字符串、十六进制数字、多行字符串等。在牺牲了一点效率的情况下，可以为用户提供更好的体验。(权衡利弊下，我们认为这点效率牺牲是值得的)

在这种输入形式下，我们通常只需要用到三个条目——`data`，`layout`和`config`:

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
该示例实现了为图表添加表头(标题为`Test`)，添加`[10, 20, 30, 40]`到饼图，控制图表不可变。

更多修改方式可以参照[JSON](#JSON输入格式)。

# 配置
打开您书的 `book.toml`，在此处我们可以添加配置:
```toml
[preprocessor.plotly]
after = ["links"]
```

在添加配置前，我们需要先确定配置的一些解析原则:

1. 所有配置名都是 `kebab-case` 命名的
2. 不正确的配置名会被忽略
3. 正确但类型错误的配置将会导致所有配置解析失败，所有配置将会被默认配置代替，同时返回类似于下面的报错:
```shell
Illegal config format for 'preprocessor.mdbook-plotly': unknown variant `plotlyhtml`, expected `plotly-html`
 |  in `output-type`
```
4. 当配置找不到时(这通常是因为没有使用`preprocessor.plotly`，因为该条目存在意味着配置存在，如果出现其他状况，请提出一个issue)，您将得到一条警告:
```shell
Custom config not found; using default configuration.
```

接下来是所有可用的配置:
```toml
[preprocessor.plotly]
after = ["links"]
# 下面内容中给出的都是默认选项

# 输出格式，这代表着代码块最后输出的结果
# 此结果字符串对应的是一个enum(对应内容可在[输出格式](#输出格式)中看到)，因而不在此范围内的字符串将导致解析失败
output-type = "plotly-html"

# 输入格式，这代表着代码块内填充数据的格式
# 此结果字符串对应的是一个enum(对应内容可在[输入格式](#输入格式)中看到)，因而不在此范围内的字符串将导致解析失败
input_type = "json-input"

# 使用本地js源
# 应为true或false(默认)
offline_js_sources = false
```

# 输入格式
输入格式决定了您可以使用怎样的方法来控制图表。它们现在由配置 `input_type` 控制，具体的可选项如下: `json-input`，`sandbox-script`。

考虑到输入格式的控制方法较复杂，下面一一进行介绍:

## JSON输入格式
该格式允许您通过JSON来控制图表。

我们采用自己的反序列化逻辑。最好的方法是参照下面的可用条目，但如果您有比较需要的条目，可以提出一个issue，我们将尝试在新版本中添加此条目。

### 文档理解须知
```json5
{
    // 我们会使用像这样的注释，帮助您理解
    // (如果您需要这个功能，请提出一个issue)
    data?: [
        // `?`说明此条目是可选的
        {
            type: "pie",  // 不带`?`，说明此条目必须存在
            // 部分情况下，如果一个条目为特殊值(或者一个条目是存在的状态)，与其关联的条目必须存在
            // 通常，我们会使用文档提醒您
            values: [usize; usize]  // 当`type`是`pie`时，`values`必须存在
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

### 类型

#### 基本类型
JSON提供了几种基本类型，其他所有的复合类型，都是由这几种基本类型延伸而来的。

##### 1. 对象
```json5
// 大括号表示对象
{}
// 其中可以用引号表示一个键名，`:`后面跟上具体内容，合起来成为一个条目
{
    "example": "wiki",
    // 也可以尾随逗号和省略引号的键名
    example2: true,
}
// 在文档中，键名后面可以跟一个`?`，具体内容则改为类型
{
    "example"?: String
}
// 部分情况下，具体内容仍为字面表达量(而不是类型名称)，说明此类型被限制在这些范围内
{
    "example1"?: "a",
    // 另外，文档中使用`|`，表示此条目对于这两者均可
    "example2"?: "a" | "b",
    "example": String | bool
}
```

##### 2. 列表
```json5
// 中括号表示列表
[]
// 其中可以用逗号隔开，输入内容
[1, 2, 3, 4]
// 文档中通常所有的内容必须是同一个类型的
// 下面这种表达法中`;`前面为类型，后面为数量
// 数量的具体格式可以参见后面内容
[String; 6]
// 有时候也可以为不同类型，但是每一位的类型都将被确定
[String, usize, String | bool, bool]
```

##### 3. 数量
```json5
// 不带引号的一个数字即为数量
1
// 文档中数量可以为以下几种类型
// 理论上它们有最大大小限制，超出限制可能是未定义的(可能直接解析失败，也有可能数字具体值不确定)
// 下面所有的内容都以一个类型带上一个示例的形式展示

// 非负整数
usize
0
// 整数
isize
-1
// 浮点数
f64
0.1

// 另外也有可能是一串范围
// 表示从0~6的任意浮点数(其中不包含6)
0..6f64

// 同时，文档中也会使用`|`
0..6usize | 10..1000f64

// 特别的，我们还有一类大小受限的数字

// 范围在[0, 255]的整数
u8
```

##### 4. 字符串
```json5
// 用引号引起来的一串内容即为字符串
"str"
// 文档中用String表示
String
// 大部分枚举使用字符串的形式，通常利用`|`
"a" | "b" | c"
```

##### 5. 布尔值
```json5
// 表示真或假的值，可以为不带引号的true或false
true
false
// 文档中一般只用bool表示
bool
// 或者与其他类型结合
String | bool
```

#### 通用复杂类型
- **Rgb**

_定义_:
```json5
// 一个特别格式的字符串，其中u8是可以更改的数字部分
// NOTE: 如果字符串格式不合理，将会导致解析失败
Rgb: "rgb(u8, u8, u8)"
```
_示例_:
```json5
"rgb(0, 0, 0)"
```

- **Rgba**

_定义_:
```json5
// 一个特别格式的字符串，其中u8和f64是可以更改的数字部分
// NOTE: 如果字符串格式不合理，将会导致解析失败
Rgba: "rgba(u8, u8, u8, f64)"
```
_示例_:
```json5
"rgba(0, 0, 0, 0.0)"
```

- **Color**
Color是一个特别复杂的通用类型，它由多个部分组合而成。

_定义_:
```json5
// 以下各个可选选项，每次只能选择一个存在(多个存在将导致解析失败，并且至少需要一个存在)
Color: {
    // 一组特别命名的颜色
    named_color?: NamedColor,
    // 从 R、G 和 B 通道构建有效 RGB 颜色
    rgb_color?: Rgb,
    // 从 R、G、B 和 A 通道构建有效 RGBA 颜色
    rgba_color?: Rgba,
}

// 跨浏览器兼容的预定义颜色(CSS的定义: https://www.w3schools.com/cssref/css_colors.php)
// NOTE: 采用lowercase命名，如AliceBlue应当写作aliceblue
// NOTE: 这是一个非常长的枚举，因而没有写出所有内容，但如果内容不存在，将会解析失败
NamedColor: String
```

_示例_:
```json5
{ named_color: "aliceblue" }
```
```json5
{ rgba: "rgba(0, 0, 0, 0)" }
```

### 映射与生成器

`map` 字段提供了一个映射表，可以在图表定义的其他地方通过 `map.key` 语法引用。这允许重复使用数据并通过内置生成器生成复杂值。

映射值可以是原始数据(任何 JSON 值)或生成器对象。生成器对象具有一个 `type` 字段表示生成算法，以及额外的参数。

#### 生成器类型

所有生成器对象必须包含 `type` 字段。支持以下生成器类型:

- **`raw`** — 直接传递数据，不做更改。

  _参数:_
  ```json5
  {
      type: "raw",
      data: T  // 任意可直接使用的值
  }
  ```
  _示例:_
  ```json5
  { type: "raw", data: [1, 2, 3] }
  ```

- **`g-number-list`** — 通过对一个整数范围内的每个整数求值表达式来生成数字列表。

  _参数:_
  ```json5
  {
      type: "g-number-list",
      begin: usize,   // 起始索引(包含)
      end: usize,     // 结束索引(不包含)
      expr: String    // 使用变量 `i` 的算术表达式
  }
  ```
  表达式使用 [fasteval](https://crates.io/crates/fasteval) 库求值；变量 `i`(作为 `f64`)在表达式中可用。

  _示例:_
  ```json5
  { type: "g-number-list", begin: 0, end: 3, expr: "i * 2" }
  // 得到 [0.0, 2.0, 4.0]
  ```

- **`g-number`** — 求值一个常数算术表达式。

  _参数:_
  ```json5
  {
      type: "g-number",
      expr: String    // 算术表达式(无变量)
  }
  ```
  _示例:_
  ```json5
  { type: "g-number", expr: "2 + 3 * 4" }
  // 得到 14.0
  ```

- **`g-range`** — 生成浮点数的算术级数。

  _参数:_
  ```json5
  {
      type: "g-range",
      begin: f64,     // 第一个值(包含)
      end: f64,       // 上界(不包含)
      step?: f64      // 步长(默认为 1.0，必须为正数)
  }
  ```
  _示例:_
  ```json5
  { type: "g-range", begin: 0.0, end: 5.0, step: 1.0 }
  // 得到 [0.0, 1.0, 2.0, 3.0, 4.0]
  ```

- **`g-repeat`** — 将给定值重复指定次数。

  _参数:_
  ```json5
  {
      type: "g-repeat",
      value: T,       // 任意 JSON 值
      count: usize    // 重复次数
  }
  ```
  _示例:_
  ```json5
  { type: "g-repeat", value: 42.0, count: 3 }
  // 得到 [42.0, 42.0, 42.0]
  ```

- **`g-linear`** — 在 `begin` 和 `end` 之间(包含两端点)线性生成 `count` 个值。

  _参数:_
  ```json5
  {
      type: "g-linear",
      begin: f64,
      end: f64,
      count: usize    // 必须为正数
  }
  ```
  如果 `count` 为 1，结果为 `[begin]`。否则步长为 `(end - begin) / (count - 1)`。

  _示例:_
  ```json5
  { type: "g-linear", begin: 0.0, end: 1.0, count: 5 }
  // 得到 [0.0, 0.25, 0.5, 0.75, 1.0]
  ```

#### 使用映射

映射条目在其他地方通过前缀 `map.` 引用。例如，如果映射包含键 `myrange`，你可以在任何接受 `DataPack<T>` 的字段(大多数数组和数字字段)中使用 `"map.myrange"`。

_完整示例:_

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

### Map格式
```json5
map: {
    // ...
}
```
Map的形式相当自由，如:
```json5
map: {
    a: "Hello",
    b: 1,
    c: false,
    // 后出现的会覆盖前面的
    c: true,
}
```
可以通过 `key: any` 来轻松定义，其中 `any` 必须是非`Object`类型，否则会优先进行特殊处理。

使用时必须`key`已被定义，然后使用 `"map.key"` 来读取，例如:
```json5
{
    map: {
        title: "Example",
        show_legend: false,
        height: 10,
    },
    layout: {
        // String类型优先识别为Map，然后再识别为String
        title: "map.title",
        show_legend: "map.show_legend",
        height: "map.height",
    }
}
```

### Layout格式
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

### Config格式
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
}
```

### Data-bar
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
}
```

### Data-candlestick
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

### Data-density_mapbox
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
### Data-histogram
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
}
```

### Data-ohlc
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

### Data-image
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

### Data-pie
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
}
```

### Data-sankey
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

### Data-scatter_geo
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
}
```

### Data-scatter_mapbox
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
}
```

### Data-scatter
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
}
```

### Data-table
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

## SandboxScript输入格式
> [!WARNING]
> **该方法已被弃用**，现在使用此方法将会导致输出一条警告和一条调试信息，并且返回默认图表。

该格式允许您通过一个脚本，它将会被放在本地的沙盒环境下运行，运行完后生成对应的图表对象。

# 输出格式
输出格式控制了图表最后的输出结果是HTML还是SVG亦或是其他的内容。这些结果各有优异，我们将选择权交给用户。

输出格式需要您在全局配置中进行操作，具体请见[配置](#配置)。

| 原始名称 | 格式化名称 | 效果 | 其他注意事项 |
|--------|--------|--------|--------|
| **PlotlyHtml** | `plotly-html`  | 输出一个 `<div>` 和配套控制逻辑的 `<script>` | 对不支持 HTML 的 Markdown 解析器可能产生兼容性问题，并且对客户端渲染不太友好 |
| **PlotlySvg**  | `plotly-svg`   | **TODO** | 当前未实现；在本地完成大部分渲染，但构建时间可能较长 |
