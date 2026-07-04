import AppKit
import Foundation
import Quartz
import WebKit

// Quick Look Preview Extension for mddiff.
//
// Loaded by macOS when the user Space-previews a supported markdown file (see
// QLSupportedContentTypes in Info.plist). We render the source into HTML by
// bundling markdown-it.min.js + preview.html + preview.css inside the .appex
// and letting WKWebView run the JS locally. This keeps the QuickLook output
// visually close to mddiff's in-app Preview view without duplicating the
// TypeScript pipeline into Swift.
//
// The class must be @objc-visible for macOS's NSExtensionPrincipalClass loader
// to resolve it via `NSClassFromString("MddiffQuickLook.PreviewProvider")`.
@objc(PreviewProvider)
final class PreviewProvider: NSViewController, QLPreviewingController, WKNavigationDelegate {
    private var webView: WKWebView!
    // WKWebView loads asynchronously; QuickLook expects us to signal readiness
    // once the preview has actually painted, otherwise the user sees a blank
    // frame before the content lands. We stash the QuickLook completion
    // handler here and fire it from the WKNavigation delegate.
    private var pendingHandler: ((Error?) -> Void)?

    override func loadView() {
        let config = WKWebViewConfiguration()
        let webView = WKWebView(frame: .zero, configuration: config)
        webView.navigationDelegate = self
        // Let the QuickLook chrome show through if our CSS ever fails to
        // paint a body background — better than a white flash.
        webView.setValue(false, forKey: "drawsBackground")
        self.webView = webView
        self.view = webView
    }

    func preparePreviewOfFile(
        at url: URL,
        completionHandler handler: @escaping (Error?) -> Void
    ) {
        do {
            let markdown = try String(contentsOf: url, encoding: .utf8)
            let html = try renderHTML(markdown: markdown)
            guard let resourceURL = Bundle.main.resourceURL else {
                throw NSError(
                    domain: "MddiffQuickLook",
                    code: 2,
                    userInfo: [NSLocalizedDescriptionKey: "resource URL missing"]
                )
            }
            pendingHandler = handler
            // baseURL is the extension's Resources dir so <script src="markdown-it.min.js">
            // and <link href="preview.css"> resolve against bundled assets.
            webView.loadHTMLString(html, baseURL: resourceURL)
        } catch {
            handler(error)
        }
    }

    private func renderHTML(markdown: String) throws -> String {
        guard let templateURL = Bundle.main.url(forResource: "preview", withExtension: "html") else {
            throw NSError(
                domain: "MddiffQuickLook",
                code: 1,
                userInfo: [NSLocalizedDescriptionKey: "preview.html not found in bundle"]
            )
        }
        let template = try String(contentsOf: templateURL, encoding: .utf8)
        // Emit the source as a valid JS string literal by round-tripping
        // through JSON. Handles quotes, backslashes, control chars,
        // non-ASCII — all the things a naive escape would miss.
        let data = try JSONEncoder().encode(markdown)
        let encoded = String(data: data, encoding: .utf8) ?? "\"\""
        return template.replacingOccurrences(of: "{{MARKDOWN_JSON}}", with: encoded)
    }

    // MARK: - WKNavigationDelegate

    func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
        pendingHandler?(nil)
        pendingHandler = nil
    }

    func webView(_ webView: WKWebView, didFail navigation: WKNavigation!, withError error: Error) {
        pendingHandler?(error)
        pendingHandler = nil
    }

    func webView(
        _ webView: WKWebView,
        didFailProvisionalNavigation navigation: WKNavigation!,
        withError error: Error
    ) {
        pendingHandler?(error)
        pendingHandler = nil
    }
}
