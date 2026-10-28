//! PortfolioMiniChart — equity curve vs benchmark mini chart.
//!
//! Faithful port of OpenTerminalUI's `components/home/PortfolioMiniChart.tsx`
//! (plan-1.md §3 2.3): dual-line SVG plotting the portfolio trajectory against
//! the benchmark overlay (NIFTY 50), with compact axis labels, crosshair hover
//! inspection and keyboard stepping. Reuses the sparkline geometry kernel from
//! `honba-kao-ui-core` and the `ot-home-widget-chart-*` class contract so the
//! Milestone 5 stylesheet ports cleanly.

use leptos::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

use honba_kao_ui_core::DEFAULT_AREA_COLOR;
use honba_kao_ui_core::DEFAULT_BENCHMARK_COLOR;
use honba_kao_ui_core::DEFAULT_LINE_COLOR;
use honba_kao_ui_core::build_sparkline_geometry;
#[cfg(target_arch = "wasm32")]
use honba_kao_ui_core::get_nearest_point_index;

/// Chart padding in SVG units (reference: 12).
pub const CHART_PADDING: f64 = 12.0;
pub const DEFAULT_CHART_WIDTH: f64 = 360.0;
pub const DEFAULT_CHART_HEIGHT: f64 = 196.0;

/// One sample on the curve: a bare value or a labelled point (e.g. a date).
#[derive(Debug, Clone, PartialEq)]
pub enum ChartPoint {
    Value(f64),
    Labeled { label: String, value: f64 },
}

impl ChartPoint {
    pub fn value(&self) -> f64 {
        match self {
            Self::Value(value) => *value,
            Self::Labeled { value, .. } => *value,
        }
    }
}

/// A finite, labelled sample ready for geometry and axis rendering.
#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedPoint {
    pub label: String,
    pub value: f64,
}

/// Bare values get `Point N` labels; non-finite values are dropped.
pub fn normalize_chart_points(points: &[ChartPoint]) -> Vec<NormalizedPoint> {
    points
        .iter()
        .enumerate()
        .map(|(index, point)| {
            let label = match point {
                ChartPoint::Value(_) => format!("Point {}", index + 1),
                ChartPoint::Labeled { label, .. } => label.clone(),
            };
            NormalizedPoint {
                label,
                value: point.value(),
            }
        })
        .filter(|point| point.value.is_finite())
        .collect()
}

/// Y axis captions: max, mid, min (formatted).
pub fn build_y_axis_labels(min: f64, max: f64, formatter: impl Fn(f64) -> String) -> Vec<String> {
    if !min.is_finite() || !max.is_finite() {
        return Vec::new();
    }
    let mid = min + (max - min) / 2.0;
    vec![formatter(max), formatter(mid), formatter(min)]
}

/// X axis captions: first, middle, last point labels.
pub fn build_x_axis_labels(points: &[NormalizedPoint]) -> Vec<String> {
    if points.is_empty() {
        return Vec::new();
    }
    let middle_index = (points.len() - 1) / 2;
    vec![
        points[0].label.clone(),
        points[middle_index].label.clone(),
        points[points.len() - 1].label.clone(),
    ]
}

/// Indian-locale grouping (`1234567.89` -> `"12,34,568"`), matching the
/// reference's `en-IN` default formatter with no fractional digits.
pub fn format_inr_grouping(value: f64) -> String {
    if !value.is_finite() {
        return "--".to_string();
    }
    let rounded = value.round();
    let is_negative = rounded < 0.0;
    let digits = (rounded.abs() as u64).to_string();
    let len = digits.len();
    let grouped = if len <= 3 {
        digits
    } else {
        let (head, tail) = digits.split_at(len - 3);
        let mut groups = Vec::new();
        let head_len = head.len();
        let mut start = 0;
        if head_len % 2 == 1 {
            groups.push(&head[..1]);
            start = 1;
        }
        while start < head_len {
            groups.push(&head[start..(start + 2).min(head_len)]);
            start += 2;
        }
        groups.push(tail);
        groups.join(",")
    };
    if is_negative {
        format!("-{grouped}")
    } else {
        grouped
    }
}

#[component]
pub fn PortfolioMiniChart(
    points: Vec<ChartPoint>,
    #[prop(optional)] benchmark_points: Option<Vec<ChartPoint>>,
    #[prop(optional, default = DEFAULT_CHART_WIDTH)] width: f64,
    #[prop(optional, default = DEFAULT_CHART_HEIGHT)] height: f64,
    #[prop(optional, into, default = "Portfolio performance chart".to_string())] aria_label: String,
    #[prop(optional, into, default = "No portfolio performance data available".to_string())]
    empty_label: String,
    #[prop(optional)] value_formatter: Option<Callback<f64, String>>,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let normalized = normalize_chart_points(&points);
    let empty_benchmark: Vec<ChartPoint> = Vec::new();
    let normalized_benchmark =
        normalize_chart_points(benchmark_points.as_deref().unwrap_or(&empty_benchmark));

    let values: Vec<f64> = normalized.iter().map(|p| p.value).collect();
    let benchmark_values: Vec<f64> = normalized_benchmark.iter().map(|p| p.value).collect();
    let geometry = build_sparkline_geometry(&values, width, height, CHART_PADDING);
    let benchmark_geometry =
        build_sparkline_geometry(&benchmark_values, width, height, CHART_PADDING);

    let formatter = value_formatter.unwrap_or_else(|| Callback::new(format_inr_grouping));
    let axis_formatter = formatter;

    if normalized.is_empty() || geometry.points.is_empty() {
        let chart_class = format!("ot-home-widget-chart {class}").trim().to_string();
        return view! {
            <div class=chart_class role="img" aria-label=empty_label>
                <div class="ot-home-widget-empty">{empty_label.clone()}</div>
            </div>
        }
        .into_any();
    }

    let total_points = geometry.points.len();
    let (active_index, set_active_index) = signal(Some(total_points.saturating_sub(1)));

    let points_for_active = geometry.points.clone();
    let active_point = Memo::new(move |_| {
        let idx = active_index.get()?;
        points_for_active.get(idx).cloned()
    });

    let labels_for_active = normalized.clone();
    let active_label = Memo::new(move |_| {
        let idx = active_index.get()?;
        labels_for_active.get(idx).map(|p| p.label.clone())
    });

    let y_axis_labels =
        build_y_axis_labels(geometry.min, geometry.max, move |v| axis_formatter.run(v));
    let x_axis_labels = build_x_axis_labels(&normalized);

    let on_keydown = move |ev: web_sys::KeyboardEvent| match ev.key().as_str() {
        "ArrowLeft" => {
            ev.prevent_default();
            set_active_index.update(|curr| {
                let cur = curr.unwrap_or(total_points.saturating_sub(1));
                *curr = Some(cur.saturating_sub(1));
            });
        }
        "ArrowRight" => {
            ev.prevent_default();
            set_active_index.update(|curr| {
                let cur = curr.unwrap_or(0);
                *curr = Some((cur + 1).min(total_points.saturating_sub(1)));
            });
        }
        _ => {}
    };

    let on_focus = move |_| set_active_index.set(Some(total_points.saturating_sub(1)));

    let on_mousemove = move |ev: web_sys::MouseEvent| {
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

    let gradient_id = format!(
        "portfolio-mini-chart-grad-{}",
        (width as u64) ^ (height as u64) ^ total_points as u64
    );

    let grid_lines = [0.2f64, 0.5, 0.8]
        .into_iter()
        .map(|ratio| {
            let y = CHART_PADDING + (height - CHART_PADDING * 2.0) * ratio;
            view! {
                <line
                    class="ot-home-widget-chart-grid-line"
                    x1=CHART_PADDING
                    x2=width - CHART_PADDING
                    y1=y
                    y2=y
                />
            }
        })
        .collect_view();

    let benchmark_line = (!benchmark_geometry.line_path.is_empty()).then(|| {
        view! {
            <path
                d=benchmark_geometry.line_path
                class="ot-home-widget-chart-benchmark"
                fill="none"
                stroke=DEFAULT_BENCHMARK_COLOR
                stroke-width="1.2"
            />
        }
    });

    let crosshair = move || {
        active_point.get().map(|pt| {
            view! {
                <line
                    class="ot-home-widget-chart-crosshair"
                    x1=pt.x
                    x2=pt.x
                    y1=CHART_PADDING
                    y2=height - CHART_PADDING
                />
                <circle
                    class="ot-home-widget-chart-marker"
                    cx=pt.x
                    cy=pt.y
                    r="3.2"
                    fill=DEFAULT_LINE_COLOR
                />
            }
        })
    };

    let tooltip_label = move || active_label.get().unwrap_or_default();
    let tooltip_value = move || formatter.run(active_point.get().map(|p| p.value).unwrap_or(0.0));

    let current_value = normalized.last().map(|p| p.value).unwrap_or_default();
    let current_label = formatter.run(current_value);
    let has_benchmark = !normalized_benchmark.is_empty();
    let benchmark_value = normalized_benchmark
        .last()
        .map(|p| p.value)
        .unwrap_or_default();
    let benchmark_label = formatter.run(benchmark_value);

    let figure_class = format!("ot-home-widget-chart {class}").trim().to_string();

    view! {
        <figure class=figure_class role="group" aria-label=aria_label.clone()>
            <div
                class="ot-home-widget-chart-frame"
                tabindex="0"
                aria-label=format!("{aria_label} interaction surface")
                on:focus=on_focus
                on:keydown=on_keydown
                on:mousemove=on_mousemove
            >
                <div class="ot-home-widget-chart-axis ot-home-widget-chart-axis-y" aria-hidden="true">
                    {y_axis_labels
                        .into_iter()
                        .map(|label| view! { <span>{label}</span> })
                        .collect_view()}
                </div>

                <div class="ot-home-widget-chart-tooltip">
                    <span class="ot-home-widget-chart-tooltip-label">{tooltip_label}</span>
                    <span class="ot-home-widget-chart-tooltip-value">{tooltip_value}</span>
                </div>

                <svg
                    class="ot-home-widget-chart-svg"
                    viewBox=format!("0 0 {width} {height}")
                    preserveAspectRatio="none"
                    aria-hidden="true"
                >
                    <defs>
                        <linearGradient id=gradient_id.clone() x1="0" x2="0" y1="0" y2="1">
                            <stop offset="0%" stop-color=DEFAULT_AREA_COLOR stop-opacity="1" />
                            <stop offset="100%" stop-color=DEFAULT_AREA_COLOR stop-opacity="0" />
                        </linearGradient>
                    </defs>

                    {grid_lines}

                    {benchmark_line}

                    <path d=geometry.area_path fill=format!("url(#{gradient_id})") />
                    <path d=geometry.line_path fill="none" stroke=DEFAULT_LINE_COLOR stroke-width="1.8" />
                    {crosshair}
                </svg>
            </div>

            <figcaption class="ot-home-widget-chart-meta">
                <div class="ot-home-widget-chart-axis ot-home-widget-chart-axis-x" aria-hidden="true">
                    {x_axis_labels
                        .into_iter()
                        .map(|label| view! { <span>{label}</span> })
                        .collect_view()}
                </div>
                <div class="ot-home-widget-chart-summary">
                    <span>"Current " {current_label}</span>
                    <Show when=move || has_benchmark>
                        <span>"Benchmark " {benchmark_label.clone()}</span>
                    </Show>
                </div>
            </figcaption>
        </figure>
    }
    .into_any()
}
