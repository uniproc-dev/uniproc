processes-connecting = Connecting...
processes-agent-gave-up = Can't reach the agent. Still trying.
processes-failed = Failed to load processes: { $error }
processes-status-processes = { $count ->
    [one] process
   *[other] processes
    }
processes-status-apps = { $count ->
    [one] app
   *[other] apps
    }
processes-status-background = background
processes-status-services = { $count ->
    [one] service
   *[other] services
    }
processes-status-kernel = kernel
processes-status-wsl = WSL
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
processes-settings-reset = Reset
processes-settings-shown-on = On
processes-settings-shown-off = Off

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
processes-wsl-note-rows = Inside, environments and processes carry the Linux agent's numbers. One environment per PID namespace, its processes flush with it.
processes-wsl-note-memory = Memory: Windows compresses and reclaims the VM's pages, so its cost can be lower than what Linux reports; the guest's page cache counts as free inside Linux yet stays held by Windows, so it can also be higher.
processes-category-background-microsoft = Background processes (Microsoft)
processes-category-windows-service = Services
processes-category-windows-kernel = Windows kernel
