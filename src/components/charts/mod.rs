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

/// Render a serialized ECharts option (JSON) into a div by `id` using the
/// vendored global `echarts`, under a shared dark "nordic" theme.
///
/// `echarts.min.js` loads asynchronously, so the first call after hydration
/// may run before it exists — we retry until it's ready. Resize is bound once
/// per element.
pub fn render_echarts(_id: &str, _option_json: &str) {
    #[cfg(target_arch = "wasm32")]
    let (id, option_json) = (_id, _option_json);
    // Only run on the client (wasm). During SSR eval is a no-op anyway, but
    // guarding here avoids unnecessary work.
    #[cfg(target_arch = "wasm32")]
    {
        let script = format!(
            r#"
            (function() {{
              var THEME = {{
                color: ['#5ef2a6','#34d3e0','#a78bfa','#f5c451','#fb7185','#43e08a','#7aa2f7','#e0aaff','#9ae6b4'],
                backgroundColor: 'transparent',
                textStyle: {{ fontFamily: 'IBM Plex Mono, monospace', color: '#8a9bb0' }},
                title: {{ textStyle: {{ color: '#e7eef6' }} }},
                legend: {{ textStyle: {{ color: '#8a9bb0' }}, inactiveColor: '#3a4756' }},
                tooltip: {{
                  backgroundColor: 'rgba(10,14,20,0.96)',
                  borderColor: '#1e2b39', borderWidth: 1,
                  padding: [8, 12],
                  textStyle: {{ color: '#e7eef6', fontFamily: 'IBM Plex Mono, monospace', fontSize: 12 }},
                  axisPointer: {{ lineStyle: {{ color: '#2a3b4d' }}, crossStyle: {{ color: '#2a3b4d' }} }}
                }},
                categoryAxis: {{
                  axisLine: {{ lineStyle: {{ color: '#1e2b39' }} }},
                  axisLabel: {{ color: '#8a9bb0' }},
                  axisTick: {{ show: false }}, splitLine: {{ show: false }}
                }},
                valueAxis: {{
                  axisLine: {{ show: false }},
                  axisLabel: {{ color: '#8a9bb0' }},
                  axisTick: {{ show: false }},
                  splitLine: {{ lineStyle: {{ color: 'rgba(30,43,57,0.55)' }} }}
                }}
              }};
              function draw() {{
                var el = document.getElementById('{id}');
                if (!el) {{ setTimeout(draw, 50); return; }}
                if (typeof echarts === 'undefined') {{ setTimeout(draw, 80); return; }}
                if (!window.__nordicThemeReg) {{ echarts.registerTheme('nordic', THEME); window.__nordicThemeReg = true; }}
                var chart = echarts.getInstanceByDom(el) || echarts.init(el, 'nordic');
                chart.setOption({option_json}, true);
                if (!el.__rb) {{
                  el.__rb = true;
                  window.addEventListener('resize', function() {{
                    var c = echarts.getInstanceByDom(el);
                    if (c) c.resize();
                  }});
                }}
              }}
              draw();
            }})();
            "#
        );
        let _ = dioxus::prelude::document::eval(&script);
    }
}
