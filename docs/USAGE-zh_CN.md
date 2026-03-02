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
        - [图表主格式](#[图表主格式])
        - [Layout格式](#Layout格式)
        - [Config格式](#Config格式)
        - [Data-pie](#Data-pie)
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

我们采用自己的反序列化逻辑。大部分情况下，你可以使用JavaScript中创建`Plot`提供的json那样的格式，但并不一定总是有效，并且我们也可能会比原始格式增多一些内容。最好的方法还是参照下面的可用条目，但如果您有比较需要的条目，可以提出一个issue，我们将尝试在新版本中添加此条目。

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
Rgb: "rgb(usize, usize, usize)"
```
_示例_:
```json5
{
    layout: {
        plot_background_color: "rgb(255, 255, 255)",
    }
}
```

### 图表主格式
```json5
{
    // 构建映射表，用来在下面的内容中填充映射
    // 暂不稳定，其结构可能在最近的几个版本中经常变更
    map?: Map,

    // 图表的布局
    layout?: Layout,

    // 图表的数据
    data?: [Data; usize],

    // 图表的配置
    config?: Configuration,
}
```

> [!WARNING]
> 下面内容仍需补充。如果您有意愿，可以提出一个PR。

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
    colorway?: [Rgb; usize],
    // 图表的背景颜色
    plot_background_color?: Rgb,
    // 分离器
    separators?: String,

    // 说明器具体设置
    legend?: {
        // 此部分暂未添加注释
        background_color?: Rgb,
        border_color?: Rgb,
        border_width?: usize,
        x?: f64,
        y?: f64,
        trace_group_gap?: usize,
        title?: String,
    },

    // 图表边缘具体设置
    margin?: {     
        // 此部分暂未添加注释
        left?: usize,
        right?: usize,
        top?: usize,
        bottom?: usize,
        pad?: usize,
        auto_expand?: bool
    },
}
```

### Config格式
```json5
// 此部分暂未添加注释
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
    send_data?: bool,
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
    // 填充区域的颜色（Rgba 格式，如 "rgba(255, 0, 0, 0.5)"）
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
