# mdbook-plotly
[![Crates.io](https://img.shields.io/crates/v/mdbook-plotly.svg?style=for-the-badge&logo=rust&color=orange)](https://crates.io/crates/mdbook-plotly)
[![CI Status](https://img.shields.io/github/actions/workflow/status/TickPoints/mdbook-plotly/ci.yml?branch=main&style=for-the-badge&logo=github-actions)](https://github.com/TickPoints/mdbook-plotly/actions)
[![License](https://img.shields.io/crates/l/mdbook-plotly?style=for-the-badge&logo=mit)](LICENSE)

**[English](README.md)**  **中文**

## 项目概述
> [!WARNING]
> 该项目尚没有稳定的发行版，只有仍然在迭代的预览版，不推荐使用到正式项目中。

`mdbook-plotly` 是一个 **mdbook** 预处理器，可在生成 HTML 文档前，将带有特殊标记(`plot`或`plotly`)的代码块转换为图表。

它可以解析以特定格式(现阶段支持JSON5)组织的图表定义，并按照所配置的输出方式输出(例如HTML)。专为技术文档设计，让图表与 Markdown 文本共存、可复现。

## 开始使用

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

### 生成图表
在您所需要的地方添加一个代码块，如下:
~~~markdown
```plot
{}
```
~~~

更多内容可以查看[使用手册](USAGE-zh_CN.md)。

## 许可证
采用 `MIT` 开源许可，完整条款请参阅 [LICENSE](LICENSE) 文件。

## 贡献指南
我们欢迎任何形式的贡献！请：
- 在提交 Pull Request 前，先通过 Issue 提出功能建议或问题讨论
- 确保所有测试通过（`cargo test`）且代码已格式化（`cargo fmt`）
- 使用清晰的提交信息
- 提交 Pull Request 到 `dev` 分支
