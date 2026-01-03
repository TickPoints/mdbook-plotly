mod code_handler;
#[cfg(feature = "plotly-svg-handler")]
mod plotly_svg_handler;

use crate::preprocessor::config::PreprocessorConfig;
use log::{error, warn};
use mdbook_preprocessor::book::Chapter;
use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};

const PULLDOWN_CMARK_OPTIONS: pulldown_cmark::Options = pulldown_cmark::Options::all();

pub fn handle(chapter: &mut Chapter, config: &PreprocessorConfig) {
    let events = Parser::new_ext(&chapter.content, PULLDOWN_CMARK_OPTIONS);

    let mut code = String::with_capacity(10);
    let mut in_target_code = false;
    let mut new_events = Vec::with_capacity(10);

    for event in events {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(ref lang))) => {
                if matches!(lang.as_ref(), "plotly" | "plot") {
                    in_target_code = true;
                    code.clear();
                } else {
                    new_events.push(event);
                }
            }
            Event::Text(ref text) => {
                if in_target_code {
                    code.push_str(text);
                } else {
                    new_events.push(event);
                }
            }
            Event::End(TagEnd::CodeBlock) => {
                if !in_target_code {
                    new_events.push(event);
                    continue;
                }
                in_target_code = false;
                let ready_code = std::mem::take(&mut code);
                match handle_plotly(ready_code, config) {
                    Ok(event) => new_events.push(event),
                    Err(message) => warn!(
                        "An error occurred during processing in Chapter {}:\n{}",
                        chapter.name, message
                    ),
                }
            }
            _ => new_events.push(event),
        }
    }
    let mut new_content = String::with_capacity(chapter.content.len());
    if let Err(e) = pulldown_cmark_to_cmark::cmark_with_options(
        new_events.into_iter(),
        &mut new_content,
        pulldown_cmark_to_cmark::Options::default(),
    ) {
        error!(
            "Processing failed during cmark. {} Chapter will use the original content. \nInterError: {:#?}",
            chapter.name, e
        );
    } else {
        chapter.content = new_content;
    }
}

pub fn handle_plotly(
    code: String,
    config: &PreprocessorConfig,
) -> Result<Event<'_>, Box<dyn std::error::Error>> {
    let ready_code = code_handler::handle(code, &config.input_type);
    use crate::preprocessor::config::PlotlyOutputType;
    let result = match config.output_type {
        PlotlyOutputType::PlotlySvg => plotly_svg_handler::handle(ready_code)?,
    };
    Ok(result)
}
