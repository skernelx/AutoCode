import AppKit

func renderSFSymbol(name: String, pointSize: CGFloat, outputSize: Int, filename: String) {
    let size = NSSize(width: outputSize, height: outputSize)
    let image = NSImage(size: size, flipped: false) { rect in
        guard let sfImage = NSImage(systemSymbolName: name, accessibilityDescription: nil) else {
            print("  ✗ SF Symbol '\(name)' not found, skipping")
            return false
        }
        let config = NSImage.SymbolConfiguration(pointSize: pointSize, weight: .semibold)
            .applying(.init(paletteColors: [.white]))
        guard let configured = sfImage.withSymbolConfiguration(config) else { return false }
        let symbolSize = configured.size
        let x = (rect.width - symbolSize.width) / 2
        let y = (rect.height - symbolSize.height) / 2
        configured.draw(in: NSRect(x: x, y: y, width: symbolSize.width, height: symbolSize.height))
        return true
    }
    guard let tiffData = image.tiffRepresentation,
          let bitmap = NSBitmapImageRep(data: tiffData),
          let pngData = bitmap.representation(using: .png, properties: [:]) else { return }
    try! pngData.write(to: URL(fileURLWithPath: filename))
}

let basePath = "/Users/wf/VSCode/AutoCode/src-tauri/icons/candidates"

// Create candidates directory
try? FileManager.default.createDirectory(atPath: basePath, withIntermediateDirectories: true)

let candidates = [
    ("checkmark.message", "01_checkmark_message"),
    ("checkmark.message.fill", "02_checkmark_message_fill"),
    ("ellipsis.message", "03_ellipsis_message"),
    ("ellipsis.message.fill", "04_ellipsis_message_fill"),
    ("lock.shield", "05_lock_shield"),
    ("checkmark.shield", "06_checkmark_shield"),
    ("key.fill", "07_key_fill"),
    ("number.square", "08_number_square"),
    ("rectangle.and.text.magnifyingglass", "09_rect_text_search"),
    ("text.bubble", "10_text_bubble"),
    ("bubble.left.and.text.bubble.right", "11_bubble_pair"),
    ("clipboard", "12_clipboard"),
    ("doc.text.magnifyingglass", "13_doc_search"),
    ("person.badge.key", "14_person_key"),
    ("shield.lefthalf.filled.badge.checkmark", "15_shield_check"),
]

for (symbol, filename) in candidates {
    print("Generating \(symbol)...")
    renderSFSymbol(name: symbol, pointSize: 32, outputSize: 64, filename: "\(basePath)/\(filename).png")
}

print("\nDone! Check \(basePath)/ for all candidates")
