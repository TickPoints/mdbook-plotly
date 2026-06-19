use super::until::Color;
use crate::code_handler::parse_context::ParseContext;
use crate::{translate, translate_enum_with_config, translate_with_config};
use anyhow::{Result, anyhow};
use plotly::{Configuration, Layout};
use serde_json::Value;

pub fn parse_config_obj(
    config_obj: &mut Value,
    context: &ParseContext<'_>,
) -> Result<Configuration> {
    use plotly::configuration::{DisplayModeBar, DoubleClick};

    let config = translate! {
        Configuration::new(),
        config_obj,
        context.map(),
        context.map_eval(),
        (static_plot, bool),
        (typeset_math, bool),
        (editable, bool),
        (autosizable, bool),
        (fill_frame, bool),
        (frame_margins, f64),
        (scroll_zoom, bool),
        (show_axis_drag_handles, bool),
        (show_axis_range_entry_boxes, bool),
        (show_tips, bool),
        (show_link, bool),
        (send_data, bool),
        (show_edit_in_chart_studio, bool),
        (double_click_delay, usize),
        (queue_length, usize),
        (display_logo, bool),
        (watermark, bool),
    }?;

    let config = translate_enum_with_config! {
        config,
        config_obj,
        context.map(),
        context.map_eval(),
        (display_mode_bar, {
            "hover"  => DisplayModeBar::Hover,
            "true"   => DisplayModeBar::True,
            "false"  => DisplayModeBar::False,
        }),
        (double_click, {
            "false"         => DoubleClick::False,
            "reset"         => DoubleClick::Reset,
            "autosize"      => DoubleClick::AutoSize,
            "reset+autosize"=> DoubleClick::ResetAutoSize,
        }),
    }?;

    Ok(config)
}

pub fn parse_layout_obj(layout_obj: &mut Value, context: &ParseContext<'_>) -> Result<Layout> {
    use plotly::layout::{
        ClickMode, DragMode, GroupClick, HoverMode, ItemClick, ItemSizing, Legend, Margin,
        TraceOrder, VAlign,
    };

    let layout = translate! {
        Layout::new(),
        layout_obj,
        context.map(),
        context.map_eval(),
        (title, String),
        (show_legend, bool),
        (auto_size, bool),
        (height, usize),
        (width, usize),
        (colorway, Vec<Color>),
        (plot_background_color, Color),
        (paper_background_color, Color),
        (separators, String),
        (bar_gap, f64),
        (bar_group_gap, f64),
        (box_gap, f64),
        (box_group_gap, f64),
    }?;

    let layout = translate_enum_with_config! {
        layout,
        layout_obj,
        context.map(),
        context.map_eval(),
        (hover_mode, {
            "x"          => HoverMode::X,
            "y"          => HoverMode::Y,
            "closest"    => HoverMode::Closest,
            "false"      => HoverMode::False,
            "x unified"  => HoverMode::XUnified,
            "y unified"  => HoverMode::YUnified,
        }),
        (drag_mode, {
            "zoom"      => DragMode::Zoom,
            "pan"       => DragMode::Pan,
            "select"    => DragMode::Select,
            "lasso"     => DragMode::Lasso,
            "orbit"     => DragMode::Orbit,
            "turntable" => DragMode::Turntable,
            "false"     => DragMode::False,
        }),
        (click_mode, {
            "event"        => ClickMode::Event,
            "select"       => ClickMode::Select,
            "none"         => ClickMode::None,
        }),
    }?;

    let layout = if let Some(legend_obj) = layout_obj.get_mut("legend")
        && legend_obj.is_object()
    {
        let legend = translate_with_config! {
            Legend::new(),
            legend_obj,
            context.map(),
            context.map_eval(),
            (background_color, Color),
            (border_color, Color),
            (border_width, usize),
            (x, f64),
            (y, f64),
            (trace_group_gap, usize),
            (item_width, usize),
            (title, String),
        }?;

        let legend = translate_enum_with_config! {
            legend,
            legend_obj,
            context.map(),
            context.map_eval(),
            (trace_order, {
                "reversed"        => TraceOrder::Reversed,
                "grouped"         => TraceOrder::Grouped,
                "reversed+grouped"=> TraceOrder::ReversedGrouped,
                "normal"          => TraceOrder::Normal,
            }),
            (item_sizing, {
                "trace"    => ItemSizing::Trace,
                "constant" => ItemSizing::Constant,
            }),
            (item_click, {
                "toggle"       => ItemClick::Toggle,
                "toggleothers" => ItemClick::ToggleOthers,
                "false"        => ItemClick::False,
            }),
            (item_double_click, {
                "toggle"       => ItemClick::Toggle,
                "toggleothers" => ItemClick::ToggleOthers,
                "false"        => ItemClick::False,
            }),
            (valign, {
                "top"    => VAlign::Top,
                "middle" => VAlign::Middle,
                "bottom" => VAlign::Bottom,
            }),
            (group_click, {
                "toggleitem" => GroupClick::ToggleItem,
                "togglegroup"=> GroupClick::ToggleGroup,
            }),
        }?;

        layout.legend(legend)
    } else {
        layout
    };

    let layout = if let Some(margin_obj) = layout_obj.get_mut("margin")
        && margin_obj.is_object()
    {
        let margin = translate_with_config! {
            Margin::new(),
            margin_obj,
            context.map(),
            context.map_eval(),
            (left, usize),
            (right, usize),
            (top, usize),
            (bottom, usize),
            (pad, usize),
            (auto_expand, bool)
        }?;
        layout.margin(margin)
    } else {
        layout
    };

    use plotly::common::Font;
    let layout = if let Some(font_obj) = layout_obj.get_mut("font")
        && font_obj.is_object()
    {
        let font = translate_with_config! {
            Font::new(),
            font_obj,
            context.map(),
            context.map_eval(),
            (family, String),
            (size, usize),
            (color, Color),
        }?;
        layout.font(font)
    } else {
        layout
    };

    use plotly::layout::ColorAxis;
    let layout = if let Some(ca_obj) = layout_obj.get_mut("coloraxis")
        && ca_obj.is_object()
    {
        let ca = translate_with_config! {
            ColorAxis::new(),
            ca_obj,
            context.map(),
            context.map_eval(),
            (cmin, f64),
            (cmax, f64),
            (cmid, f64),
            (auto_color_scale, bool),
            (reverse_scale, bool),
            (show_scale, bool),
        }?;
        layout.color_axis(ca)
    } else {
        layout
    };

    let layout = if let Some(axis_obj) = layout_obj.get_mut("xaxis")
        && axis_obj.is_object()
    {
        let axis = parse_axis_obj(axis_obj, context)?;
        layout.x_axis(axis)
    } else {
        layout
    };

    let layout = if let Some(axis_obj) = layout_obj.get_mut("yaxis")
        && axis_obj.is_object()
    {
        let axis = parse_axis_obj(axis_obj, context)?;
        layout.y_axis(axis)
    } else {
        layout
    };

    let layout = parse_named_axes(layout, layout_obj, context, "x")?;
    let layout = parse_named_axes(layout, layout_obj, context, "y")?;

    Ok(layout)
}

fn parse_axis_obj(
    axis_obj: &mut Value,
    context: &ParseContext<'_>,
) -> Result<plotly::layout::Axis> {
    use crate::code_handler::until::DataPack;
    use plotly::layout::{Axis, AxisType};

    let axis = translate_with_config! {
        Axis::new(),
        axis_obj,
        context.map(),
        context.map_eval(),
        (title, String),
        (show_grid, bool),
        (show_line, bool),
        (zero_line, bool),
        (visible, bool),
        (anchor, String),
        (overlaying, String),
        (range, Vec<Option<f64>>),
        (color, Color),
        (line_color, Color),
        (grid_color, Color),
        (tick_prefix, String),
        (tick_suffix, String),
        (tick_format, String),
        (hover_format, String),
        (category_array, Vec<String>),
        (fixed_range, bool),
        (scale_anchor, String),
        (auto_margin, bool),
        (show_tick_labels, bool),
    }?;

    let axis = translate_enum_with_config! {
        axis,
        axis_obj,
        context.map(),
        context.map_eval(),
        (category_order, {
            "trace"               => plotly::layout::CategoryOrder::Trace,
            "category-ascending"  => plotly::layout::CategoryOrder::CategoryAscending,
            "category-descending" => plotly::layout::CategoryOrder::CategoryDescending,
            "array"               => plotly::layout::CategoryOrder::Array,
            "total-ascending"     => plotly::layout::CategoryOrder::TotalAscending,
            "total-descending"    => plotly::layout::CategoryOrder::TotalDescending,
            "min-ascending"       => plotly::layout::CategoryOrder::MinAscending,
            "min-descending"      => plotly::layout::CategoryOrder::MinDescending,
            "max-ascending"       => plotly::layout::CategoryOrder::MaxAscending,
            "max-descending"      => plotly::layout::CategoryOrder::MaxDescending,
            "sum-ascending"       => plotly::layout::CategoryOrder::SumAscending,
            "sum-descending"      => plotly::layout::CategoryOrder::SumDescending,
            "mean-ascending"      => plotly::layout::CategoryOrder::MeanAscending,
            "mean-descending"     => plotly::layout::CategoryOrder::MeanDescending,
            "median-ascending"    => plotly::layout::CategoryOrder::MedianAscending,
            "median-descending"   => plotly::layout::CategoryOrder::MedianDescending,
        }),
    }?;

    let axis = if let Some(v) = axis_obj.get_mut("type") {
        let data = serde_json::from_value::<DataPack<String>>(v.take())
            .map_err(|e| anyhow!("Failed to deserialize axis `type`: {}", e))?;
        let s = data
            .unwrap_from_context(context)
            .map_err(|e| anyhow!("Failed to unwrap DataPack for axis `type`: {}", e))?;
        let at = match s.as_str() {
            "-" | "linear" => AxisType::Linear,
            "log" => AxisType::Log,
            "date" => AxisType::Date,
            "category" => AxisType::Category,
            "multicategory" => AxisType::MultiCategory,
            other => return Err(anyhow!("Invalid axis type: '{}'", other)),
        };
        axis.type_(at)
    } else {
        axis
    };

    Ok(axis)
}

fn parse_named_axes(
    layout: Layout,
    layout_obj: &mut Value,
    context: &ParseContext<'_>,
    prefix: &str,
) -> Result<Layout> {
    let mut layout = layout;
    for i in 2..=8 {
        let json_key = format!("{}axis{}", prefix, i);
        let Some(axis_obj) = layout_obj.get_mut(json_key.as_str()) else {
            continue;
        };
        if !axis_obj.is_object() {
            continue;
        }
        let axis = parse_axis_obj(axis_obj, context)?;
        if prefix == "x" {
            layout = match i {
                2 => layout.x_axis2(axis),
                3 => layout.x_axis3(axis),
                4 => layout.x_axis4(axis),
                5 => layout.x_axis5(axis),
                6 => layout.x_axis6(axis),
                7 => layout.x_axis7(axis),
                8 => layout.x_axis8(axis),
                _ => unreachable!(),
            };
        } else {
            layout = match i {
                2 => layout.y_axis2(axis),
                3 => layout.y_axis3(axis),
                4 => layout.y_axis4(axis),
                5 => layout.y_axis5(axis),
                6 => layout.y_axis6(axis),
                7 => layout.y_axis7(axis),
                8 => layout.y_axis8(axis),
                _ => unreachable!(),
            };
        }
    }
    Ok(layout)
}
