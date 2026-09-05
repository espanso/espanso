function getTauriInvoke() {
  return (
    window.__TAURI__?.core?.invoke ??
    window.__TAURI__?.tauri?.invoke ??
    window.__TAURI__?.invoke ??
    null
  );
}

async function invokeTauri(command, payload = {}) {
  const invokeFn = getTauriInvoke();
  if (!invokeFn) {
    throw new Error("Tauri bridge is unavailable.");
  }

  return invokeFn(command, payload);
}

const state = {
  matches: [],
  selectedId: null,
  filePath: "",
  toastTimer: null,
};

const el = {
  filePath: document.getElementById("filePath"),
  useDefaultBtn: document.getElementById("useDefaultBtn"),
  reloadBtn: document.getElementById("reloadBtn"),
  newBtn: document.getElementById("newBtn"),
  saveBtn: document.getElementById("saveBtn"),
  deleteBtn: document.getElementById("deleteBtn"),
  triggerInput: document.getElementById("triggerInput"),
  replaceInput: document.getElementById("replaceInput"),
  labelInput: document.getElementById("labelInput"),
  matchList: document.getElementById("matchList"),
  selectionBadge: document.getElementById("selectionBadge"),
  statsFile: document.getElementById("statsFile"),
  statsCount: document.getElementById("statsCount"),
  toast: document.getElementById("toast"),
};

function showToast(message, isError = false) {
  if (state.toastTimer) {
    clearTimeout(state.toastTimer);
  }

  el.toast.textContent = message;
  el.toast.classList.add("show");
  el.toast.classList.toggle("error", isError);
  state.toastTimer = setTimeout(() => {
    el.toast.classList.remove("show");
    el.toast.classList.remove("error");
  }, 2800);
}

function setSelection(matchRecord) {
  state.selectedId = matchRecord?.id ?? null;
  el.deleteBtn.disabled = !matchRecord;

  if (!matchRecord) {
    el.triggerInput.value = "";
    el.replaceInput.value = "";
    el.labelInput.value = "";
    el.selectionBadge.textContent = "Creating new";
    el.saveBtn.textContent = "Add match";
  } else {
    el.triggerInput.value = matchRecord.trigger ?? "";
    el.replaceInput.value = matchRecord.replace ?? "";
    el.labelInput.value = matchRecord.label ?? "";
    el.selectionBadge.textContent = `Editing #${matchRecord.id}`;
    el.saveBtn.textContent = "Update match";
  }

  renderList();
}

function renderList() {
  el.matchList.innerHTML = "";
  el.statsCount.textContent = String(state.matches.length);
  el.statsFile.textContent = state.filePath || "—";

  if (!state.matches.length) {
    const empty = document.createElement("li");
    empty.className = "match-item";
    empty.innerHTML = `<p class="match-trigger">No matches found</p><p class="match-replace">Create your first shortcut using the editor.</p>`;
    el.matchList.appendChild(empty);
    return;
  }

  for (const matchRecord of state.matches) {
    const li = document.createElement("li");
    li.className = "match-item";
    if (state.selectedId === matchRecord.id) {
      li.classList.add("active");
    }

    const chips = [];
    if (matchRecord.label) {
      chips.push(`<span class="chip">${escapeHtml(matchRecord.label)}</span>`);
    }
    if (matchRecord.has_multiple_triggers) {
      chips.push(`<span class="chip">multi-trigger source</span>`);
    }

    li.innerHTML = `
      <p class="match-trigger">${escapeHtml(matchRecord.trigger)}</p>
      <p class="match-replace">${escapeHtml(matchRecord.replace || "(empty replacement)")}</p>
      <div class="match-meta">${chips.join("")}</div>
    `;

    li.addEventListener("click", () => {
      setSelection(matchRecord);
    });

    el.matchList.appendChild(li);
  }
}

function escapeHtml(input) {
  return String(input)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
}

async function refreshMatches() {
  const filePath = el.filePath.value.trim();
  if (!filePath) {
    showToast("Please provide a match file path.", true);
    return;
  }

  state.filePath = filePath;
  try {
    state.matches = await invokeTauri("list_matches", { filePath });
    setSelection(null);
    showToast("Matches loaded.");
  } catch (err) {
    showToast(`Failed to load matches: ${err}`, true);
  }
}

function getFormPayload() {
  const trigger = el.triggerInput.value.trim();
  const replace = el.replaceInput.value.trim();
  const label = el.labelInput.value.trim();

  return {
    file_path: state.filePath,
    trigger,
    replace,
    label: label || null,
  };
}

async function saveCurrent() {
  if (!state.filePath) {
    showToast("Load a match file first.", true);
    return;
  }

  const payload = getFormPayload();
  if (!payload.trigger || !payload.replace) {
    showToast("Trigger and replacement are required.", true);
    return;
  }

  try {
    if (state.selectedId == null) {
      await invokeTauri("add_match", { payload });
      showToast("Match added.");
    } else {
      await invokeTauri("update_match", {
        payload: {
          ...payload,
          id: state.selectedId,
        },
      });
      showToast("Match updated.");
    }

    await refreshMatches();
  } catch (err) {
    showToast(`Save failed: ${err}`, true);
  }
}

async function removeSelected() {
  if (state.selectedId == null) {
    showToast("Select a match to delete.", true);
    return;
  }

  try {
    await invokeTauri("remove_match", {
      payload: {
        file_path: state.filePath,
        id: state.selectedId,
      },
    });
    showToast("Match removed.");
    await refreshMatches();
  } catch (err) {
    showToast(`Delete failed: ${err}`, true);
  }
}

async function setDefaultFilePath() {
  try {
    const filePath = await invokeTauri("get_default_match_file");
    el.filePath.value = filePath;
    state.filePath = filePath;
    await refreshMatches();
  } catch (err) {
    showToast(`Failed to resolve default file path: ${err}`, true);
  }
}

function wireEvents() {
  el.reloadBtn.addEventListener("click", refreshMatches);
  el.useDefaultBtn.addEventListener("click", setDefaultFilePath);
  el.newBtn.addEventListener("click", () => setSelection(null));
  el.saveBtn.addEventListener("click", saveCurrent);
  el.deleteBtn.addEventListener("click", removeSelected);
}

async function start() {
  wireEvents();
  try {
    await setDefaultFilePath();
  } catch (err) {
    showToast(`Startup failed: ${err}`, true);
  }
}

start().catch((err) => {
  showToast(`Startup failed: ${err}`, true);
});
