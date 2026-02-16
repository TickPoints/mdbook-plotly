# mdbook-plotly
[![Crates.io](https://img.shields.io/crates/v/mdbook-plotly.svg?style=for-the-badge&logo=rust&color=orange)](https://crates.io/crates/mdbook-plotly)
[![Build Status](https://img.shields.io/github/actions/workflow/status/TickPoints/mdbook-plotly/release.yml&style=for-the-badge&logo=github-actions)](https://github.com/TickPoints/mdbook-plotly/actions)
[![License](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)

**English**  **[中文](README-zh_CN.md)**

## Project Overview
> [!WARNING]
> This project has not yet reached a stable release; only preview versions—still under active development—are currently available. It is not recommended for use in production projects.

`mdbook-plotly` is a preprocessor for **mdbook**, which converts specially tagged code blocks (`plot` or `plotly`) into interactive charts before generating the final HTML documentation.

It parses chart definitions structured in specific formats (currently supporting JSON5) and renders them according to the configured output format (e.g., HTML). Designed specifically for technical documentation, it enables charts to coexist seamlessly with Markdown text while ensuring reproducibility.

## Getting Started

### Installation
1. Using Cargo
```shell
cargo install mdbook-plotly
# If you use binstall:
cargo binstall mdbook-plotly
```

Alternatively, download the latest available release for your system from the [Releases](https://github.com/TickPoints/mdbook-plotly/releases) page on GitHub, then add the application’s directory to your system’s PATH environment variable.

2. Add the following to your `book.toml`:
```toml
[preprocessor.plotly]
after = ["links"]
```

### Generating Charts
Insert a code block where you want the chart to appear, like this:
~~~markdown
```plot
{}
```
~~~

For more details, refer to the [User Guide](docs/USAGE.md).  

## License  
This project is licensed under the **MIT License**. For full terms, see the [LICENSE](LICENSE) file.  

## Contributing  
We welcome contributions of all kinds! Please follow these guidelines:  
- Before submitting a Pull Request, open an Issue first to propose new features or discuss bugs.  
- Ensure all tests pass (`cargo test`) and code is properly formatted (`cargo fmt`).  
- Use clear, descriptive commit messages.  
- Submit Pull Requests targeting the `dev` branch.
