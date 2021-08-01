display alert "Espanso wasn't able to automatically disable secure input. Sometimes locking and unlocking the screen helps, do you want to try?" buttons {"No", "Yes"} default button "Yes"
if button returned of result = "No" then
  return "no"
else
  if button returned of result = "Yes" then
    return "yes"
  end if
end if