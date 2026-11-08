# Third-party notices

mdview bundles the following front-end libraries into the executable so that
Markdown documents can be rendered fully offline.

| Component | Version | License | Project |
| --- | --- | --- | --- |
| highlight.js | 11.10.0 | BSD-3-Clause | https://github.com/highlightjs/highlight.js |
| KaTeX | 0.16.11 | MIT | https://github.com/KaTeX/KaTeX |
| Mermaid | 11.4.1 | MIT | https://github.com/mermaid-js/mermaid |
| auto-render (KaTeX contrib) | 0.16.11 | MIT | https://github.com/KaTeX/KaTeX |
| Turndown | 7.2.0 | MIT | https://github.com/mixmark-io/turndown |
| turndown-plugin-gfm | 1.0.2 | MIT | https://github.com/mixmark-io/turndown-plugin-gfm |

Rust dependencies are resolved by Cargo and listed in `Cargo.lock`. The main
GUI/rendering dependencies are:

| Crate | License | Project |
| --- | --- | --- |
| wry | Apache-2.0 OR MIT | https://github.com/tauri-apps/wry |
| tao | Apache-2.0 OR MIT | https://github.com/tauri-apps/tao |
| pulldown-cmark | MIT | https://github.com/raphlinus/pulldown-cmark |
| webview2-com | MIT OR Apache-2.0 | https://github.com/wravery/webview2-rs |
| rfd | MIT | https://github.com/PolyMeilex/rfd |
| encoding_rs | Apache-2.0 OR MIT | https://github.com/hsivonen/encoding_rs |
| base64 | MIT OR Apache-2.0 | https://github.com/marshallpierce/rust-base64 |

The libraries are used unmodified except for concatenation/minification as
published by their official CDN builds. Their license headers are retained in
the bundled files where present.

The full license texts of the bundled front-end libraries are reproduced in
the [`licenses/`](licenses/) directory:

| Component | License text |
| --- | --- |
| KaTeX / auto-render | [`licenses/KaTeX.txt`](licenses/KaTeX.txt) |
| Mermaid | [`licenses/Mermaid.txt`](licenses/Mermaid.txt) |
| Turndown | [`licenses/Turndown.txt`](licenses/Turndown.txt) |
| turndown-plugin-gfm | [`licenses/turndown-plugin-gfm.txt`](licenses/turndown-plugin-gfm.txt) |
| highlight.js | [`licenses/highlight.js.txt`](licenses/highlight.js.txt) |
