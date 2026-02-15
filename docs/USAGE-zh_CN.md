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
    - [SandScript](#SandScript输入格式)
- [输出格式](#输出格式)

# 开始使用

## 下载
您可以从GitHub页面的[Releases](https://github.com/TickPoints/mdbook-plotly/releases)根据您的系统信息下载最新可用版本。

## 安装
您需要把此应用程序所在路径添加到系统的 `Path` 环境变量下，或者您可以先记下此程序所在的绝对路径或对于您的书的相对路径，这在接下来可以使用。

然后，请在您书的 `book.toml` 添加如下内容:
```toml
[preprocessor.plotly]
after = ["links"]
```
如果您没有添加环境变量，而是使用路径的话，则可以:
```toml
[preprocessor.plotly]
after = ["links"]
# 添加`command`条目，其中填写路径，例如: `../mdbook-plotly/target/debug/mdbook-plotly`
command = "<path>"
```
然后，您就可以愉快地使用`mdbook build`等操作。

## 生成图表
在您所需要的地方添加一个代码块，如下:
~~~markdown
```plot
{}
```
~~~
这是基本的输入形式(JSON)，其他输入格式可以参考[输入格式](#输入格式)。另外用`plot`和`plotly`两个名字的代码块均可以生成图表。

> [!NOTE]
> 我们使用了`json5`，它允许注释、尾随逗号、无引号的对象键、单引号字符串、十六进制数字、多行字符串等。在牺牲了一点效率的情况下，可以为用户提供更好的体验。(权衡利弊下，我们认为这点效率牺牲是值得的)

在这种输入形式下，我们通常只需要用到两个条目——`data`和`layout`:

~~~markdown
```plot
{
    "layout": {
        "title": "Test"
    }
}
```
~~~
该示例实现了为表格添加表头(标题为`Test`)。

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

1. 对象
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

2. 列表
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

3. 数量
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

4. 字符串
```json5
// 用引号引起来的一串内容即为字符串
"str"
// 文档中用String表示
String
// 大部分枚举使用字符串的形式，通常利用`|`
"a" | "b" | c"
```

5. 布尔值
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

定义:
```json5
Rgb: "rgb(usize, usize, usize)"
```
示例:
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

### Data-pie
`pie`可以是一个`Data`。该`Data`将会被渲染为饼图。
```json5
// 此部分暂未添加注释
{
    type: "pie",

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
    text_info?: String,
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
