pub mod price_chart;
pub mod generation_pie;
pub mod forecast_line;
pub mod flow_map;

/// Monotonic id source so multiple chart instances get unique DOM ids.
pub fn next_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static N: AtomicU64 = AtomicU64::new(0);
    N.fetch_add(1, Ordering::Relaxed)
}

/// Render a serialized ECharts option (JSON) into a 100%-width div by `id`,
/// using the vendored global `echarts`.
pub fn render_echarts(id: &str, option_json: &str) {
    let script = format!(
        r#"
        (function() {{
          if (typeof echarts === 'undefined') return;
          var el = document.getElementById('{id}');
          if (!el) return;
          var chart = echarts.getInstanceByDom(el) || echarts.init(el);
          chart.setOption({option_json});
          if (!el.__echartsResizeBound) {{
            el.__echartsResizeBound = true;
            window.addEventListener('resize', function() {{
              var c = echarts.getInstanceByDom(el);
              if (c) c.resize();
            }});
          }}
        }})();
        "#
    );
    let _ = dioxus::prelude::document::eval(&script);
}
