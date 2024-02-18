tell application "System Events"
	set listOfProcesses to (bundle identifier of every process where background only is false)
end tell

return listOfProcesses