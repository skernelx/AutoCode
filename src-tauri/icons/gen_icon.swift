import AppKit

let basePath = "/Users/wf/VSCode/AutoCode/src-tauri/icons"

func savePNG(_ image: NSImage, to path: String) throws {
    guard let tiffData = image.tiffRepresentation,
          let bitmap = NSBitmapImageRep(data: tiffData),
          let pngData = bitmap.representation(using: .png, properties: [:]) else {
        throw NSError(domain: "AutoCodeIcon", code: -1, userInfo: [
            NSLocalizedDescriptionKey: "failed to encode png",
        ])
    }
    try pngData.write(to: URL(fileURLWithPath: path))
}

func drawSFSymbol(
    name: String,
    in rect: NSRect,
    pointSize: CGFloat,
    weight: NSFont.Weight,
    color: NSColor
) {
    guard let sfImage = NSImage(systemSymbolName: name, accessibilityDescription: nil) else {
        return
    }
    let config = NSImage.SymbolConfiguration(pointSize: pointSize, weight: weight)
        .applying(.init(paletteColors: [color]))
    guard let configured = sfImage.withSymbolConfiguration(config) else {
        return
    }
    configured.draw(in: rect)
}

func drawPaperLines(in rect: NSRect) {
    let lineColor = NSColor(srgbRed: 0.78, green: 0.85, blue: 0.96, alpha: 1.0)
    lineColor.setStroke()
    let stroke = NSBezierPath()
    stroke.lineWidth = max(1.0, rect.width * 0.055)

    for i in 0..<3 {
        let y = rect.maxY - rect.height * (0.24 + CGFloat(i) * 0.26)
        stroke.move(to: NSPoint(x: rect.minX + rect.width * 0.14, y: y))
        stroke.line(to: NSPoint(x: rect.maxX - rect.width * 0.14, y: y))
    }

    stroke.lineCapStyle = .round
    stroke.stroke()
}

func drawOpenEnvelope(in rect: NSRect, withShadow: Bool) {
    let w = rect.width
    let h = rect.height

    let bodyRect = NSRect(
        x: rect.minX + w * 0.08,
        y: rect.minY + h * 0.10,
        width: w * 0.84,
        height: h * 0.48
    )
    let flapBaseY = bodyRect.maxY - h * 0.02
    let apex = NSPoint(x: rect.midX, y: rect.minY + h * 0.88)

    let flapPath = NSBezierPath()
    flapPath.move(to: NSPoint(x: bodyRect.minX, y: flapBaseY))
    flapPath.line(to: apex)
    flapPath.line(to: NSPoint(x: bodyRect.maxX, y: flapBaseY))
    flapPath.close()

    let paperRect = NSRect(
        x: rect.minX + w * 0.22,
        y: rect.minY + h * 0.36,
        width: w * 0.56,
        height: h * 0.36
    )
    let paperPath = NSBezierPath(
        roundedRect: paperRect,
        xRadius: w * 0.04,
        yRadius: w * 0.04
    )

    let bodyPath = NSBezierPath(
        roundedRect: bodyRect,
        xRadius: w * 0.06,
        yRadius: w * 0.06
    )

    if withShadow {
        NSGraphicsContext.saveGraphicsState()
        let shadow = NSShadow()
        shadow.shadowBlurRadius = w * 0.03
        shadow.shadowOffset = NSSize(width: 0, height: -h * 0.018)
        shadow.shadowColor = NSColor(white: 0, alpha: 0.22)
        shadow.set()
        bodyPath.fill()
        NSGraphicsContext.restoreGraphicsState()
    }

    let flapGradient = NSGradient(
        colorsAndLocations:
            (NSColor(srgbRed: 1.00, green: 0.75, blue: 0.40, alpha: 1.0), 0.0),
            (NSColor(srgbRed: 1.00, green: 0.57, blue: 0.35, alpha: 1.0), 1.0)
    )!
    flapGradient.draw(in: flapPath, angle: -90)

    NSColor(white: 0.98, alpha: 1.0).setFill()
    paperPath.fill()
    drawPaperLines(in: paperRect)

    let bodyGradient = NSGradient(
        colorsAndLocations:
            (NSColor(srgbRed: 1.00, green: 0.49, blue: 0.42, alpha: 1.0), 0.0),
            (NSColor(srgbRed: 0.96, green: 0.25, blue: 0.42, alpha: 1.0), 1.0)
    )!
    bodyGradient.draw(in: bodyPath, angle: -90)

    let leftFold = NSBezierPath()
    leftFold.move(to: NSPoint(x: bodyRect.minX, y: bodyRect.maxY))
    leftFold.line(to: NSPoint(x: bodyRect.midX, y: bodyRect.midY - bodyRect.height * 0.05))
    leftFold.line(to: NSPoint(x: bodyRect.minX, y: bodyRect.minY))
    leftFold.close()
    NSColor(white: 1.0, alpha: 0.18).setFill()
    leftFold.fill()

    let rightFold = NSBezierPath()
    rightFold.move(to: NSPoint(x: bodyRect.maxX, y: bodyRect.maxY))
    rightFold.line(to: NSPoint(x: bodyRect.midX, y: bodyRect.midY - bodyRect.height * 0.05))
    rightFold.line(to: NSPoint(x: bodyRect.maxX, y: bodyRect.minY))
    rightFold.close()
    NSColor(white: 0.0, alpha: 0.08).setFill()
    rightFold.fill()

    let edgeColor = NSColor(srgbRed: 0.49, green: 0.14, blue: 0.24, alpha: 0.38)
    edgeColor.setStroke()
    flapPath.lineWidth = max(1.0, w * 0.012)
    bodyPath.lineWidth = max(1.0, w * 0.012)
    flapPath.stroke()
    bodyPath.stroke()
}

func renderAppMasterIcon(size: Int, output: String) throws {
    let canvas = NSSize(width: size, height: size)
    let image = NSImage(size: canvas, flipped: false) { rect in
        NSGraphicsContext.current?.imageInterpolation = .high

        let pad = rect.width * 0.055
        let iconRect = rect.insetBy(dx: pad, dy: pad)
        let radius = iconRect.width * 0.24
        let bgPath = NSBezierPath(roundedRect: iconRect, xRadius: radius, yRadius: radius)

        NSGraphicsContext.saveGraphicsState()
        bgPath.addClip()
        let bgGradient = NSGradient(
            colorsAndLocations:
                (NSColor(srgbRed: 0.16, green: 0.48, blue: 1.0, alpha: 1.0), 0.0),
                (NSColor(srgbRed: 0.24, green: 0.70, blue: 1.0, alpha: 1.0), 0.52),
                (NSColor(srgbRed: 0.51, green: 0.42, blue: 1.0, alpha: 1.0), 1.0)
        )!
        bgGradient.draw(in: bgPath, angle: -35)

        let glow = NSGradient(
            colorsAndLocations:
                (NSColor(white: 1.0, alpha: 0.30), 0.0),
                (NSColor(white: 1.0, alpha: 0.0), 1.0)
        )!
        let glowRect = NSRect(
            x: iconRect.minX - iconRect.width * 0.20,
            y: iconRect.midY - iconRect.height * 0.12,
            width: iconRect.width * 1.45,
            height: iconRect.height * 0.95
        )
        glow.draw(in: NSBezierPath(ovalIn: glowRect), relativeCenterPosition: .zero)
        NSGraphicsContext.restoreGraphicsState()

        NSColor(white: 1.0, alpha: 0.20).setStroke()
        bgPath.lineWidth = max(1.0, rect.width * 0.008)
        bgPath.stroke()

        let envelopeRect = NSRect(
            x: iconRect.minX + iconRect.width * 0.14,
            y: iconRect.minY + iconRect.height * 0.11,
            width: iconRect.width * 0.72,
            height: iconRect.height * 0.76
        )
        drawOpenEnvelope(in: envelopeRect, withShadow: true)

        return true
    }

    try savePNG(image, to: output)
    print("Wrote \(output) (\(size)x\(size))")
}

func renderTrayIcon(outputSize: Int, filename: String) throws {
    let size = NSSize(width: outputSize, height: outputSize)
    let image = NSImage(size: size, flipped: false) { rect in
        guard let ctx = NSGraphicsContext.current?.cgContext else {
            return false
        }
        let u = CGFloat(outputSize) / 22.0

        ctx.setShouldAntialias(false)
        ctx.setAllowsAntialiasing(false)
        ctx.interpolationQuality = .none

        // Solid white "open envelope" base shape
        ctx.setFillColor(NSColor.white.cgColor)
        let outer = CGMutablePath()
        outer.move(to: CGPoint(x: 3 * u, y: 13 * u))
        outer.addLine(to: CGPoint(x: 11 * u, y: 20 * u))
        outer.addLine(to: CGPoint(x: 19 * u, y: 13 * u))
        outer.addLine(to: CGPoint(x: 19 * u, y: 5 * u))
        outer.addLine(to: CGPoint(x: 3 * u, y: 5 * u))
        outer.closeSubpath()
        ctx.addPath(outer)
        ctx.fillPath()

        // Carve out details to keep the icon sharp at 22x22
        ctx.setBlendMode(.clear)

        // Opening gap at top
        let gap = CGMutablePath()
        gap.move(to: CGPoint(x: 6 * u, y: 13 * u))
        gap.addLine(to: CGPoint(x: 11 * u, y: 17 * u))
        gap.addLine(to: CGPoint(x: 16 * u, y: 13 * u))
        gap.closeSubpath()
        ctx.addPath(gap)
        ctx.fillPath()

        // Inner folds
        ctx.setLineWidth(max(1.0, 1.4 * u))
        ctx.setLineCap(.square)
        ctx.beginPath()
        ctx.move(to: CGPoint(x: 4.2 * u, y: 12.4 * u))
        ctx.addLine(to: CGPoint(x: 11 * u, y: 8.0 * u))
        ctx.move(to: CGPoint(x: 17.8 * u, y: 12.4 * u))
        ctx.addLine(to: CGPoint(x: 11 * u, y: 8.0 * u))
        ctx.strokePath()

        ctx.setBlendMode(.normal)
        return true
    }

    try savePNG(image, to: filename)
    print("Wrote \(filename) (\(outputSize)x\(outputSize))")
}

do {
    try renderAppMasterIcon(size: 1024, output: "\(basePath)/icon.png")
    try renderTrayIcon(outputSize: 22, filename: "\(basePath)/tray-icon.png")
    try renderTrayIcon(outputSize: 44, filename: "\(basePath)/tray-icon@2x.png")
    print("Done.")
} catch {
    fputs("Icon generation failed: \(error)\n", stderr)
    exit(1)
}
