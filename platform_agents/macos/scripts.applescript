-- Focus frontmost app and paste clipboard content
-- Requires Accessibility permission for System Events

tell application "System Events" to keystroke "v" using {command down}
