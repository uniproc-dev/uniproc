processes-connecting = Connecting…
processes-agent-gave-up = Can’t reach the service. Still trying.
processes-agent-outdated = The service is older than this Uniproc. Update it.
processes-failed = Failed to load processes: { $error }
processes-status-processes = { $count ->
    [one] { $count } process
   *[other] { $count } processes
    }
processes-status-apps = { $count ->
    [one] { $count } app
   *[other] { $count } apps
    }
processes-status-background = { $count } background
processes-status-services = { $count ->
    [one] { $count } service
   *[other] { $count } services
    }
processes-status-kernel = { $count } kernel
processes-status-wsl = { $count } WSL
processes-group-count = ({ $count })
processes-owned-name = { $owner } — { $name }
processes-run-new-task = Run new task
processes-gpu-engine = GPU { $adapter } - { $engine }
processes-platform-x86 = 32-bit
processes-platform-x64 = 64-bit
processes-platform-arm = ARM
processes-platform-arm64 = ARM64
processes-platform-arm64-x86 = x86 on ARM64
processes-platform-arm64-x64 = x64 on ARM64
processes-exited = Exited
processes-not-running = Not running
processes-selected = Selected: { $name } | PID { $pid }
processes-selected-exited = Selected: { $name } | PID { $pid } | Exited
processes-selected-group = Selected: { $name } | Group ({ $count })
processes-selected-linux = Selected: { $name } | PID { $pid } | { $environment }

processes-settings-title = Settings
processes-settings-columns = Columns
processes-settings-columns-description = Column order on the Processes page
processes-settings-columns-reset = Default columns
processes-settings-sections = Sections
processes-settings-sections-description = Section order on the Processes page
processes-settings-sections-reset = Default section order
processes-settings-memory = Memory values
processes-settings-memory-description = How memory use is shown on the Processes page
processes-settings-memory-values = Values
processes-settings-memory-percents = Percents
processes-settings-reset = Reset
processes-settings-shown-on = On
processes-settings-shown-off = Off

processes-menu-more-columns = More columns…
processes-menu-pin = Pin
processes-menu-unpin = Unpin
processes-menu-end-task = End task
processes-menu-suspend = Suspend
processes-menu-resume = Resume
processes-menu-open-file-location = Open file location
processes-menu-properties = Properties
processes-menu-search-online = Search online
processes-menu-switch-to = Switch to
processes-menu-minimize = Minimize
processes-menu-maximize = Maximize
processes-menu-close-window = Close window

processes-category-pinned = Pinned
processes-category-wsl = WSL
processes-wsl-namespace = PID namespace { $id }
processes-wsl-note-heading = The heading shows WSL the way Windows sees it: what the virtual machine (vmmemWSL) costs.
processes-wsl-note-rows = Inside, environments and processes carry the numbers of the service inside WSL. One environment per PID namespace, its processes flush with it.
processes-wsl-note-memory = Memory: Windows compresses and reclaims the VM's pages, so its cost can be lower than what Linux reports; the guest's page cache counts as free inside Linux yet stays held by Windows, so it can also be higher.
processes-category-background-microsoft = Background processes (Microsoft)
processes-category-windows-service = Services
processes-category-windows-kernel = Windows kernel
