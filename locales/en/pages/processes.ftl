processes-connecting = Connecting...
processes-agent-gave-up = Can't reach the agent. Still trying.
processes-failed = Failed to load processes: { $error }
processes-status = { $count ->
    [one] { $count } process
   *[other] { $count } processes
    } · { $apps ->
    [one] { $apps } app
   *[other] { $apps } apps
    } · { $background } background · { $services ->
    [one] { $services } service
   *[other] { $services } services
    } · { $kernel } kernel
processes-status-wsl = WSL { $count }
processes-status-pinned = { $count } pinned
processes-exited = Exited
processes-not-running = Not running
processes-selected = Selected: { $name } | PID { $pid }
processes-selected-exited = Selected: { $name } | PID { $pid } | Exited
processes-selected-group = Selected: { $name } | Group ({ $count })
processes-selected-linux = Selected: { $name } | PID { $pid } | { $environment }

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
