action-failure-title = { $action ->
    [kill] Couldn’t end { $target }
    [suspend] Couldn’t suspend { $target }
    [resume] Couldn’t resume { $target }
    [start] Couldn’t start { $target }
    [stop] Couldn’t stop { $target }
    [pause] Couldn’t pause { $target }
    [continue] Couldn’t resume { $target }
    [restart] Couldn’t restart { $target }
   *[other] Couldn’t change { $target }
    }
action-failure-denied = Access is denied.
action-failure-gone = It is no longer running.
action-failure-busy = Another action on it hasn’t finished yet.
action-failure-not-connected = The service isn’t connected.
action-failure-failed = Windows error { $code }.
