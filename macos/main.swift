// Branches.app: a menu bar manager for the Branches server.
//
// The app has no window. It lives in the menu bar, starts `branches-server
// --app` (or adopts one that is already running), shows whether it is up,
// and offers the few things you need: open it, stop or restart it, find your
// worlds, read the log. Everything is in this one file so CI can build it
// with a plain `swiftc`; top-level code lives here because this is main.swift.

import AppKit
import ServiceManagement

// MARK: - Settings

/// The port the app server uses (7878 unless PORT says otherwise).
let port = ProcessInfo.processInfo.environment["PORT"].flatMap { Int($0) } ?? 7878
let baseURL = "http://localhost:\(port)"
let releaseURL = "https://github.com/staplehorsey/branches/releases/download/mac-latest/Branches-mac.zip"
let home = NSHomeDirectory()
let logURL = URL(fileURLWithPath: home + "/Library/Logs/Branches.log")

/// Where worlds live: DATA_DIR if set, else the app's usual folder.
func dataDirURL() -> URL {
    if let d = ProcessInfo.processInfo.environment["DATA_DIR"], !d.isEmpty {
        return URL(fileURLWithPath: d)
    }
    return URL(fileURLWithPath: home + "/Library/Application Support/Branches")
}

/// Short requests only: a server that does not answer quickly counts as down.
let session: URLSession = {
    let c = URLSessionConfiguration.ephemeral
    c.timeoutIntervalForRequest = 2
    c.timeoutIntervalForResource = 4
    c.requestCachePolicy = .reloadIgnoringLocalCacheData
    return URLSession(configuration: c)
}()

// MARK: - App

final class AppDelegate: NSObject, NSApplicationDelegate {
    private var statusItem: NSStatusItem!
    private let menu = NSMenu()
    private let statusLine = NSMenuItem(title: "Branches is stopped", action: nil, keyEquivalent: "")
    private let residentsLine = NSMenuItem(title: "", action: nil, keyEquivalent: "")
    private var toggleItem: NSMenuItem!
    private var restartItem: NSMenuItem!
    private var loginItem: NSMenuItem!

    /// The server we started ourselves, if any. An adopted server has none.
    private var child: Process?
    private var running = false
    /// True while a start or stop is in flight, so the menu cannot double up.
    private var busy = false
    /// Set by Quit Branches, so applicationWillTerminate does not repeat the stop.
    private var quitting = false
    private var timer: Timer?

    // MARK: Launch

    func applicationDidFinishLaunching(_ notification: Notification) {
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        if let button = statusItem.button {
            // door.left.hand.open needs macOS 12; on 11 we fall back to text.
            if let img = NSImage(systemSymbolName: "door.left.hand.open", accessibilityDescription: "Branches") {
                img.isTemplate = true
                button.image = img
            } else {
                button.title = "⌂"
            }
        }
        buildMenu()
        statusItem.menu = menu

        timer = Timer.scheduledTimer(withTimeInterval: 3, repeats: true) { [weak self] _ in self?.poll() }

        // Adopt a server that is already up, otherwise start our own.
        probe { [weak self] up in
            guard let self = self else { return }
            if up {
                self.running = true
                self.refresh()
                self.openBrowser()
            } else {
                self.startServer()
            }
        }
    }

    /// Double-clicking the app again just brings the browser up.
    func applicationShouldHandleReopen(_ sender: NSApplication, hasVisibleWindows flag: Bool) -> Bool {
        openBrowser()
        return false
    }

    /// Logout, shutdown and the like: do not leave a server we started behind.
    func applicationWillTerminate(_ notification: Notification) {
        if quitting { return }
        if let c = child, c.isRunning {
            c.terminate()
            let end = Date().addingTimeInterval(2)
            while c.isRunning && Date() < end { Thread.sleep(forTimeInterval: 0.1) }
        }
    }

    // MARK: Menu

    private func buildMenu() {
        menu.autoenablesItems = false
        statusLine.isEnabled = false
        residentsLine.isEnabled = false
        residentsLine.isHidden = true
        menu.addItem(statusLine)
        menu.addItem(residentsLine)
        menu.addItem(.separator())

        menu.addItem(item("Open Branches", #selector(openBranches), key: "o"))
        toggleItem = item("Start Server", #selector(toggleServer))
        menu.addItem(toggleItem)
        restartItem = item("Restart Server", #selector(restartServer))
        menu.addItem(restartItem)
        menu.addItem(.separator())

        menu.addItem(item("Show Worlds Folder", #selector(showWorlds)))
        menu.addItem(item("Open Log", #selector(openLog)))
        loginItem = item("Launch at Login", #selector(toggleLogin))
        if #available(macOS 13.0, *) {
            loginItem.state = SMAppService.mainApp.status == .enabled ? .on : .off
        } else {
            loginItem.isHidden = true
        }
        menu.addItem(loginItem)
        menu.addItem(item("Get the Latest Version…", #selector(getLatest)))
        menu.addItem(.separator())
        menu.addItem(item("Quit Branches", #selector(quit), key: "q"))
        refresh()
    }

    private func item(_ title: String, _ action: Selector, key: String = "") -> NSMenuItem {
        let i = NSMenuItem(title: title, action: action, keyEquivalent: key)
        i.target = self
        return i
    }

    /// Brings the menu in line with `running` and `busy`.
    private func refresh() {
        toggleItem.title = running ? "Stop Server" : "Start Server"
        toggleItem.isEnabled = !busy
        restartItem.isEnabled = !busy
        if !running {
            statusLine.title = "Branches is stopped"
            residentsLine.isHidden = true
        }
    }

    // MARK: Actions

    @objc private func openBranches() { openBrowser() }

    @objc private func toggleServer() {
        if running { stopServer { } } else { startServer() }
    }

    @objc private func restartServer() {
        stopServer { [weak self] in self?.startServer(openWhenUp: false) }
    }

    @objc private func showWorlds() {
        let dir = dataDirURL()
        try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        NSWorkspace.shared.open(dir)
    }

    @objc private func openLog() {
        // An empty file is better than "no such file" if the server never ran.
        if !FileManager.default.fileExists(atPath: logURL.path) {
            FileManager.default.createFile(atPath: logURL.path, contents: nil)
        }
        NSWorkspace.shared.open(logURL)
    }

    @objc private func toggleLogin() {
        guard #available(macOS 13.0, *) else { return }
        let service = SMAppService.mainApp
        do {
            if service.status == .enabled { try service.unregister() } else { try service.register() }
        } catch {
            let alert = NSAlert()
            alert.messageText = "Could not change Launch at Login"
            alert.informativeText = error.localizedDescription
            alert.runModal()
        }
        loginItem.state = service.status == .enabled ? .on : .off
    }

    @objc private func getLatest() {
        if let url = URL(string: releaseURL) { NSWorkspace.shared.open(url) }
    }

    @objc private func quit() {
        quitting = true
        stopServer { NSApp.terminate(nil) }
    }

    // MARK: Browser

    private func openBrowser() {
        // CI sets this so the smoke test does not try to open a browser.
        if ProcessInfo.processInfo.environment["BRANCHES_NO_OPEN"] != nil { return }
        if let url = URL(string: baseURL + "/") { NSWorkspace.shared.open(url) }
    }

    // MARK: Server process

    /// Starts our own server and opens the browser once it answers.
    private func startServer(openWhenUp: Bool = true) {
        if busy || child != nil { return }
        busy = true
        refresh()
        // The login shell can be slow, so find PATH off the main thread.
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            let path = AppDelegate.searchPath()
            DispatchQueue.main.async { self?.spawn(path: path, openWhenUp: openWhenUp) }
        }
    }

    private func spawn(path: String, openWhenUp: Bool) {
        let exe = Bundle.main.bundleURL.appendingPathComponent("Contents/MacOS/branches-server")
        let p = Process()
        p.executableURL = exe
        p.arguments = ["--app"]
        var env = ProcessInfo.processInfo.environment
        env["BRANCHES_APP"] = "1"
        // We open the browser ourselves.
        env["BRANCHES_NO_OPEN"] = "1"
        env["PATH"] = path
        p.environment = env
        p.standardInput = FileHandle.nullDevice

        if let log = logHandle() {
            p.standardOutput = log
            p.standardError = log
        }
        p.terminationHandler = { [weak self] proc in
            DispatchQueue.main.async {
                guard let self = self, self.child === proc else { return }
                // Exited on its own or was stopped; either way, no restart loop.
                self.child = nil
                self.running = false
                self.busy = false
                self.refresh()
            }
        }
        do {
            try p.run()
        } catch {
            busy = false
            refresh()
            return
        }
        child = p
        waitUntilUp(tries: 40, openWhenUp: openWhenUp)
    }

    /// Checks twice a second, for about 20 seconds, for the server to answer.
    private func waitUntilUp(tries: Int, openWhenUp: Bool) {
        probe { [weak self] up in
            guard let self = self else { return }
            if up {
                self.busy = false
                self.running = true
                self.refresh()
                self.poll()
                if openWhenUp { self.openBrowser() }
            } else if tries > 0 && self.child != nil {
                DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
                    self.waitUntilUp(tries: tries - 1, openWhenUp: openWhenUp)
                }
            } else {
                // Gave up, or the child already exited.
                self.busy = false
                self.refresh()
            }
        }
    }

    /// Asks the server to quit, then waits for it. A child that is still alive
    /// after 8 seconds is terminated. `done` runs on the main thread.
    private func stopServer(done: @escaping () -> Void) {
        busy = true
        refresh()
        var req = URLRequest(url: URL(string: baseURL + "/api/app/quit")!)
        req.httpMethod = "POST"
        req.setValue("application/json", forHTTPHeaderField: "Content-Type")
        req.httpBody = "{}".data(using: .utf8)
        session.dataTask(with: req) { _, _, _ in
            DispatchQueue.main.async {
                self.waitForStop(deadline: Date().addingTimeInterval(8), done: done)
            }
        }.resume()
    }

    private func waitForStop(deadline: Date, done: @escaping () -> Void) {
        let finish = { [weak self] in
            guard let self = self else { return }
            self.running = false
            self.busy = false
            self.refresh()
            done()
        }
        let again = { [weak self] in
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.25) {
                self?.waitForStop(deadline: deadline, done: done)
            }
        }
        if let c = child {
            if !c.isRunning {
                child = nil
                finish()
            } else if Date() > deadline {
                c.terminate()
                // Give the termination a moment to land.
                DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
                    self.child = nil
                    finish()
                }
            } else {
                again()
            }
        } else {
            // An adopted server: watch the port instead.
            probe { up in
                if !up || Date() > deadline { finish() } else { again() }
            }
        }
    }

    /// Opens the log for appending, creating it if needed.
    private func logHandle() -> FileHandle? {
        let fm = FileManager.default
        try? fm.createDirectory(atPath: home + "/Library/Logs", withIntermediateDirectories: true)
        if !fm.fileExists(atPath: logURL.path) { fm.createFile(atPath: logURL.path, contents: nil) }
        guard let h = FileHandle(forWritingAtPath: logURL.path) else { return nil }
        h.seekToEndOfFile()
        return h
    }

    /// Apps started from Finder get a bare PATH. Borrow the login shell's so
    /// agents such as `claude` are found, then add the usual install spots.
    private static func searchPath() -> String {
        var path = ProcessInfo.processInfo.environment["PATH"] ?? "/usr/bin:/bin:/usr/sbin:/sbin"
        let sh = Process()
        sh.executableURL = URL(fileURLWithPath: "/bin/zsh")
        sh.arguments = ["-ilc", "echo __PATH__$PATH"]
        let pipe = Pipe()
        sh.standardOutput = pipe
        sh.standardError = FileHandle.nullDevice
        sh.standardInput = FileHandle.nullDevice
        do {
            try sh.run()
            // A shell that hangs (a prompt in .zshrc, say) must not hang us.
            DispatchQueue.global().asyncAfter(deadline: .now() + 5) { if sh.isRunning { sh.terminate() } }
            let data = pipe.fileHandleForReading.readDataToEndOfFile()
            sh.waitUntilExit()
            let text = String(data: data, encoding: .utf8) ?? ""
            let line = text.split(separator: "\n").last { $0.hasPrefix("__PATH__") }
            if let line = line {
                let found = String(line.dropFirst("__PATH__".count))
                if !found.isEmpty { path = found }
            }
        } catch {}
        let extra = ["/opt/homebrew/bin", "/usr/local/bin", home + "/.local/bin", home + "/.claude/local"]
        return ([path] + extra).joined(separator: ":")
    }

    // MARK: Status

    /// Does a Branches server answer on our port?
    private func probe(_ done: @escaping (Bool) -> Void) {
        fetch("/.well-known/branches.json") { data in
            let up = data.flatMap { String(data: $0, encoding: .utf8) }?.contains("branches/0.1") ?? false
            DispatchQueue.main.async { done(up) }
        }
    }

    private func fetch(_ path: String, _ done: @escaping (Data?) -> Void) {
        guard let url = URL(string: baseURL + path) else { return done(nil) }
        session.dataTask(with: url) { data, response, _ in
            let ok = (response as? HTTPURLResponse)?.statusCode == 200
            done(ok ? data : nil)
        }.resume()
    }

    /// Every 3 seconds: is it up, and what is going on in it? Failures are ignored.
    private func poll() {
        probe { [weak self] up in
            guard let self = self else { return }
            self.running = up
            if !up {
                // A server that went away on its own: show stopped, offer Start.
                if let c = self.child, !c.isRunning { self.child = nil }
                self.refresh()
                return
            }
            self.refresh()
            self.statusLine.title = "Branches is running"
            self.fetch("/api/worlds") { data in
                guard let data = data,
                      let worlds = try? JSONSerialization.jsonObject(with: data) as? [[String: Any]] else { return }
                let online = worlds.reduce(0) { $0 + ((($1["online"]) as? NSNumber)?.intValue ?? 0) }
                DispatchQueue.main.async {
                    if self.running && online > 0 {
                        self.statusLine.title = "Running · \(online) \(online == 1 ? "person" : "people") here"
                    }
                }
            }
            self.fetch("/api/residents") { data in
                guard let data = data,
                      let obj = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
                      let list = obj["residents"] as? [Any] else { return }
                let coins = ((obj["economy"] as? [String: Any])?["treasury"] as? NSNumber)?.intValue
                var text = "\(list.count) \(list.count == 1 ? "resident" : "residents")"
                if let coins = coins { text += " · \(coins) \(coins == 1 ? "coin" : "coins")" }
                DispatchQueue.main.async {
                    guard self.running else { return }
                    self.residentsLine.title = text
                    self.residentsLine.isHidden = false
                }
            }
        }
    }
}

// MARK: - Entry point

let app = NSApplication.shared
let delegate = AppDelegate()
app.delegate = delegate
app.setActivationPolicy(.accessory)
app.run()
