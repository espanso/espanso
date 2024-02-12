-- Activate each Window of Google Chrome and press CMD+L to focus the address bar
-- This makes it possible to "blur" any focused password field that might be keeping
-- SecureInput enabled
tell application "Google Chrome"
	activate
	-- For each window
	repeat with win in windows
		-- Bring to front
		set index of item 1 of win to 1
		delay 0.5
		-- And press CMD+L
		tell application "System Events" to keystroke "l" using command down
		delay 0.5
	end repeat
end tell