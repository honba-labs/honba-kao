//! SparklineCell — SVG sparkline with gradient fill and interactive tooltip.
//!
//! Models OpenTerminalUI's `SparklineCell.tsx`. Renders inline SVG trends with
//! optional benchmark comparison lines and accessible keyboard/focus navigation.

use leptos::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

pub const SPARKLINE_PADDING: f64 = 4.0;
pub const DEFAULT_LINE_COLOR: &str = "var(--ot-color-accent-primary, #4ade80)";
pub const DEFAULT_AREA_COLOR: &str =
    "var(--ot-color-home-widget-chart-fill-top, rgba(74, 222, 128, 0.25))";
pub const DEFAULT_BENCHMARK_COLOR: &str = "var(--ot-color-home-widget-chart-benchmark, #64748b)";
pub const DEFAULT_EMPTY_LABEL: &str = "No sparkline data available";

#[derive(Debug, Clone, PartialEq)]
pub struct SparklineGeometryPoint {
    pub index: usize,
    pub value: f64,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SparklineGeometry {
    pub area_path: String,
    pub line_path: String,
    pub max: f64,
    pub min: f64,
    pub points: Vec<SparklineGeometryPoint>,
}

pub fn sanitize_points(points: &[f64]) -> Vec<f64> {
    points.iter().copied().filter(|p| p.is_finite()).collect()
}

pub fn build_sparkline_geometry(
    raw_points: &[f64],
    width: f64,
    height: f64,
    padding: f64,
) -> SparklineGeometry {
    let points = sanitize_points(raw_points);
    let safe_width = width.max(1.0);
    let safe_height = height.max(1.0);
    let padded_width = (safe_width - padding * 2.0).max(1.0);
    let padded_height = (safe_height - padding * 2.0).max(1.0);

    if points.is_empty() {
        return SparklineGeometry {
            area_path: String::new(),
            line_path: String::new(),
            max: 0.0,
            min: 0.0,
            points: Vec::new(),
        };
    }

    let min = points.iter().copied().fold(f64::INFINITY, f64::min);
    let max = points.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let range = if (max - min).abs() < f64::EPSILON {
        1.0
    } else {
        max - min
    };

    let count = points.len();
    let chart_points: Vec<SparklineGeometryPoint> = if count == 1 {
        let v = points[0];
        vec![
            SparklineGeometryPoint {
                index: 0,
                value: v,
                x: padding,
                y: padding + padded_height / 2.0,
            },
            SparklineGeometryPoint {
                index: 1,
                value: v,
                x: padding + padded_width,
                y: padding + padded_height / 2.0,
            },
        ]
    } else {
        points
            .iter()
            .enumerate()
            .map(|(index, &value)| {
                let x = padding + (index as f64 / (count - 1) as f64) * padded_width;
                let y = padding + padded_height - ((value - min) / range) * padded_height;
                SparklineGeometryPoint { index, value, x, y }
            })
            .collect()
    };

    let line_path = chart_points
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let cmd = if i == 0 { "M" } else { "L" };
            format!("{cmd}{:.2} {:.2}", p.x, p.y)
        })
        .collect::<Vec<_>>()
        .join(" ");

    let first = &chart_points[0];
    let last = &chart_points[chart_points.len() - 1];
    let baseline = safe_height - padding;
    let area_path = format!(
        "{line_path} L {:.2} {:.2} L {:.2} {:.2} Z",
        last.x, baseline, first.x, baseline
    );

    SparklineGeometry {
        area_path,
        line_path,
        max,
        min,
        points: chart_points,
    }
}

pub fn get_nearest_point_index(
    points_len: usize,
    client_x: f64,
    rect_left: f64,
    rect_width: f64,
) -> usize {
    if points_len <= 1 || rect_width <= 0.0 {
        return 0;
    }
    let relative_x = (client_x - rect_left).clamp(0.0, rect_width);
    let last_index = (points_len - 1) as f64;
    ((relative_x / rect_width) * last_index).round() as usize
}

#[component]
pub fn SparklineCell(
    points: Vec<f64>,
    #[prop(optional, default = 132.0)] width: f64,
    #[prop(optional, default = 42.0)] height: f64,
    #[prop(optional, into, default = DEFAULT_LINE_COLOR.to_string())] color: String,
    #[prop(optional, into, default = DEFAULT_AREA_COLOR.to_string())] area_color: String,
    #[prop(optional)] benchmark_points: Option<Vec<f64>>,
    #[prop(optional, into, default = DEFAULT_BENCHMARK_COLOR.to_string())] benchmark_color: String,
    #[prop(optional, into)] class: String,
    #[prop(optional, into, default = "Sparkline trend".to_string())] aria_label: String,
    #[prop(optional, into, default = DEFAULT_EMPTY_LABEL.to_string())] empty_label: String,
    #[prop(optional, default = false)] show_tooltip: bool,
    #[prop(optional)] value_formatter: Option<Callback<f64, String>>,
) -> impl IntoView {
    let geometry = build_sparkline_geometry(&points, width, height, SPARKLINE_PADDING);
    let benchmark_geometry = build_sparkline_geometry(
        benchmark_points.as_deref().unwrap_or(&[]),
        width,
        height,
        SPARKLINE_PADDING,
    );

    let has_points = !geometry.points.is_empty();
    let interactive = show_tooltip && has_points;
    let initial_index = if interactive {
        Some(geometry.points.len().saturating_sub(1))
    } else {
        None
    };
    let (active_index, set_active_index) = signal(initial_index);

    let wrapper_class = format!("ot-home-widget-sparkline {class}")
        .trim()
        .to_string();

    if !has_points {
        let label_clone = empty_label.clone();
        return view! {
            <div
                class=wrapper_class
                data-empty="true"
                role="img"
                aria-label=empty_label
            >
                <div class="ot-home-widget-empty">{label_clone}</div>
            </div>
        }
        .into_any();
    }

    let gradient_id = format!(
        "sparkline-grad-{}",
        (width as u64) ^ (height as u64) ^ points.len() as u64
    );
    let total_points = geometry.points.len();
    let points_for_closure = geometry.points.clone();

    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        if !interactive {
            return;
        }
        let key = ev.key();
        if key == "ArrowLeft" {
            ev.prevent_default();
            set_active_index.update(|curr| {
                let cur = curr.unwrap_or(total_points.saturating_sub(1));
                *curr = Some(cur.saturating_sub(1));
            });
        } else if key == "ArrowRight" {
            ev.prevent_default();
            set_active_index.update(|curr| {
                let cur = curr.unwrap_or(0);
                *curr = Some((cur + 1).min(total_points.saturating_sub(1)));
            });
        }
    };

    let on_focus = move |_| {
        if interactive {
            set_active_index.set(Some(total_points.saturating_sub(1)));
        }
    };

    let on_blur = move |_| {
        if interactive {
            set_active_index.set(None);
        }
    };

    let on_mouseleave = move |_| {
        if interactive {
            set_active_index.set(None);
        }
    };

    let on_mousemove = move |ev: web_sys::MouseEvent| {
        if !interactive {
            return;
        }
        #[cfg(target_arch = "wasm32")]
        {
            let target = ev.current_target();
            if let Some(el) = target.and_then(|t| t.dyn_into::<web_sys::Element>().ok()) {
                let rect = el.get_bounding_client_rect();
                let idx = get_nearest_point_index(
                    total_points,
                    ev.client_x() as f64,
                    rect.left(),
                    rect.width(),
                );
                set_active_index.set(Some(idx));
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = ev;
        }
    };

    let active_point_memo = Memo::new(move |_| {
        let idx = active_index.get()?;
        points_for_closure.get(idx).cloned()
    });

    let default_formatter = Callback::new(|v: f64| -> String {
        crate::home::market_heat_strip::format_number_with_commas(v, 2)
    });
    let formatter = value_formatter.unwrap_or(default_formatter);

    let tooltip_view = move || {
        if let Some(pt) = active_point_memo.get() {
            let left_pct = (pt.x / width) * 100.0;
            let formatted_val = formatter.run(pt.value);
            Some(view! {
                <div class="ot-home-widget-tooltip" style=format!("left: {left_pct:.2}%")>
                    <span>{formatted_val}</span>
                </div>
            })
        } else {
            None
        }
    };

    let marker_color = color.clone();
    let active_markers_view = move || {
        active_point_memo.get().map(|pt| {
            view! {
                <line
                    class="ot-home-widget-sparkline-crosshair"
                    x1=pt.x
                    x2=pt.x
                    y1=SPARKLINE_PADDING
                    y2=height - SPARKLINE_PADDING
                />
                <circle
                    class="ot-home-widget-sparkline-marker"
                    cx=pt.x
                    cy=pt.y
                    r="2.8"
                    fill=marker_color.clone()
                />
            }
        })
    };

    let benchmark_line = (!benchmark_geometry.line_path.is_empty()).then(|| {
        view! {
            <path
                d=benchmark_geometry.line_path
                class="ot-home-widget-sparkline-benchmark"
                fill="none"
                stroke=benchmark_color
                stroke-width="1"
            />
        }
    });

    view! {
        <div
            class=wrapper_class
            data-interactive=if interactive { "true" } else { "false" }
            role="img"
            aria-label=aria_label
            tabindex=if interactive { 0 } else { -1 }
            on:focus=on_focus
            on:blur=on_blur
            on:keydown=on_keydown
            on:mouseleave=on_mouseleave
            on:mousemove=on_mousemove
        >
            {tooltip_view}

            <svg
                class="ot-home-widget-sparkline-svg"
                viewBox=format!("0 0 {width} {height}")
                preserveAspectRatio="none"
                aria-hidden="true"
            >
                <defs>
                    <linearGradient id=gradient_id.clone() x1="0" x2="0" y1="0" y2="1">
                        <stop offset="0%" stop-color=area_color.clone() stop-opacity="1" />
                        <stop offset="100%" stop-color=area_color stop-opacity="0" />
                    </linearGradient>
                </defs>
                {benchmark_line}
                <path d=geometry.area_path fill=format!("url(#{gradient_id})") />
                <path d=geometry.line_path fill="none" stroke=color stroke-width="1.8" />
                {active_markers_view}
            </svg>
        </div>
    }
    .into_any()
}
