activity-title = Activity
activity-came = Started
activity-went = Exited
activity-new-only = New exe only
activity-series = Group repeats
activity-span-half-minute = 30 s
activity-span-five-minutes = 5 min
activity-span-quarter = 15 min
activity-span-half-hour = 30 min
activity-span-hour = 1 h
activity-span-day = 24 h
activity-span-connected = Since connect
activity-search = Search names and command lines
activity-hover-came = started { $at } · { $lived }
activity-hover-went = exited { $at }
activity-range = { $from } – { $to } · { $came } started, { $went } exited
activity-range-lost = { $from } – { $to } · { $came } started, { $went } exited · { $lost } missed by the service
activity-history-since = History starts at { $at }
activity-history-full = History since the service connected
activity-empty = Nothing started or exited here
activity-loading = Waiting for the service

activity-from = from { $launcher }
activity-from-task = task “{ $task }”
activity-from-services = from the { $services } service
activity-first-seen = first seen
activity-still-running = still running
activity-unknown-process = Process { $pid }
activity-unknown-process-why = started before the service’s history, so its name isn’t known
activity-earlier = { $count } earlier — pick a minute on the chart to see them
activity-paused = Showing { $from } – { $to }
activity-live = Back to live
activity-series-name = { $name } × { $count }
activity-series-names-separator = {", "}
activity-series-summary = { $came } started, { $went } exited
activity-services-separator = {", "}
activity-menu-manage-groups = Manage groups…
activity-legend-other = Other
activity-legend-count = { $count }
activity-group-windows-background = Windows background
activity-pick-put-exe = Put { $program } in
activity-pick-put-under = Put what { $launcher } starts in
activity-pick-new-group = New group
activity-pick-only = Only this program
activity-pick-hide-exe = Hide this program
activity-pick-hide-folder = Hide this folder
activity-pick-hide-launcher = Hide what { $launcher } starts
activity-picked-only = Only { $what }
activity-chain-separator = {" › "}

activity-fact-command-line = Command line
activity-fact-launched-by = Launched by
activity-fact-task = Scheduled task
activity-fact-task-value = “{ $name }” · { $path }
activity-fact-parent-services = Started by services
activity-fact-folder = Folder
activity-fact-user = User
activity-fact-user-value = { $user } · session { $session }
activity-fact-user-elevated = { $user } · session { $session } · administrator
activity-fact-exit = Exit
activity-fact-exit-value = { $at } · code { $code } · lived { $lived }
activity-fact-exit-code = { $at } · code { $code }
activity-fact-unknown = Not known

activity-groups-title = Groups
activity-groups-rules = { $count ->
    [one] { $count } rule
   *[other] { $count } rules
}
activity-groups-built-in-rules = Built in · { $count ->
    [one] { $count } rule
   *[other] { $count } rules
}
activity-groups-shown-on = Shown
activity-groups-shown-off = Hidden
activity-groups-name = Group name
activity-groups-up = Move up
activity-groups-down = Move down
activity-groups-delete = Delete group
activity-groups-drop-rule = Take out of this group
activity-groups-empty = Nothing in this group yet. Right-click a row on the Activity page to put a program here.
activity-groups-order = A process in two groups goes to the upper one.
activity-groups-hidden = Hidden entirely
activity-groups-nothing-hidden = Nothing is hidden. A row's menu on the Activity page hides a program, a folder or what a program starts.
activity-groups-show-again = Show again
activity-groups-colour = Colour
activity-groups-rule-exe = { $program }
activity-groups-rule-folder = Folder { $folder }
activity-groups-rule-under = Everything under { $launcher }

activity-milliseconds = { $count } ms
activity-seconds = { $count } s
activity-minutes = { $count } min
activity-hours = { $count } h
activity-days = { $count } d
