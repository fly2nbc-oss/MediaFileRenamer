import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getVersion } from "@tauri-apps/api/app";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { open } from "@tauri-apps/plugin-dialog";
import type {
  AppState,
  FileEntry,
  PreviewEntry,
  ProgressPayload,
  RenameFormat,
  RenameResult,
  ToolsStatus,
} from "./types";

const MEDIA_EXTENSIONS = [
  "jpg", "jpeg", "png", "heic", "heif", "tiff", "tif",
  "webp", "gif", "bmp", "mp4", "mov", "avi", "mkv",
];

/** Minimal file count to show a confirmation dialog before renaming. */
const CONFIRM_LARGE_BATCH_THRESHOLD = 500;

/** Debounce delay (ms) for preview refresh on offset/format changes. */
const PREVIEW_DEBOUNCE_MS = 100;

const state: AppState = {
  files: [],
  previews: [],
  format: "YYYY_MM_DD__hhmmss",
  offsetSeconds: 0,
  createBackup: false,
  convertHeic: true,
  selectedIndices: new Set(),
  isProcessing: false,
  canUndo: false,
  tools: { exiftool: false, heif_convert: false },
};

// ── DOM refs ──

const $addBtn = document.getElementById("btn-add") as HTMLButtonElement;
const $removeBtn = document.getElementById("btn-remove") as HTMLButtonElement;
const $formatSelect = document.getElementById("format-select") as HTMLSelectElement;
const $chkBackup = document.getElementById("chk-backup") as HTMLInputElement;
const $chkHeic = document.getElementById("chk-heic") as HTMLInputElement;
const $offsetSeconds = document.getElementById("offset-seconds") as HTMLInputElement;
const $expandBtn = document.getElementById("btn-expand-offset") as HTMLButtonElement;
const $offsetExpanded = document.getElementById("offset-expanded") as HTMLElement;
const $offYears = document.getElementById("off-years") as HTMLInputElement;
const $offMonths = document.getElementById("off-months") as HTMLInputElement;
const $offDays = document.getElementById("off-days") as HTMLInputElement;
const $offHours = document.getElementById("off-hours") as HTMLInputElement;
const $offMinutes = document.getElementById("off-minutes") as HTMLInputElement;
const $offSecs = document.getElementById("off-secs") as HTMLInputElement;
const $emptyState = document.getElementById("empty-state") as HTMLElement;
const $tableWrap = document.getElementById("file-table-wrap") as HTMLElement;
const $tbody = document.getElementById("file-tbody") as HTMLTableSectionElement;
const $chkAll = document.getElementById("chk-all") as HTMLInputElement;
const $undoBtn = document.getElementById("btn-undo") as HTMLButtonElement;
const $aboutBtn = document.getElementById("btn-about") as HTMLButtonElement;
const $renameBtn = document.getElementById("btn-rename") as HTMLButtonElement;
const $statusText = document.getElementById("status-text") as HTMLElement;
const $progressOverlay = document.getElementById("progress-overlay") as HTMLElement;
const $progressFill = document.getElementById("progress-bar-fill") as HTMLElement;
const $progressText = document.getElementById("progress-text") as HTMLElement;
const $progressFile = document.getElementById("progress-file") as HTMLElement;
const $errorPanel = document.getElementById("error-panel") as HTMLElement;
const $errorTitle = document.getElementById("error-title") as HTMLElement;
const $errorList = document.getElementById("error-list") as HTMLUListElement;
const $closeErrors = document.getElementById("btn-close-errors") as HTMLButtonElement;
const $fileListContainer = document.getElementById("file-list-container") as HTMLElement;
const $aboutDialog = document.getElementById("about-dialog") as HTMLElement;
const $closeAbout = document.getElementById("btn-close-about") as HTMLButtonElement;
const $aboutVersion = document.getElementById("about-version") as HTMLElement;

// ── Init ──

async function init() {
  state.tools = await invoke<ToolsStatus>("check_tools");
  state.canUndo = await invoke<boolean>("has_undo");
  updateUndoButton();
  await hydrateAboutDialog();

  bindEvents();
  setupDragDrop();
}

async function hydrateAboutDialog() {
  try {
    $aboutVersion.textContent = await getVersion();
  } catch {
    // Keep the placeholder when runtime app metadata is unavailable.
  }
}

// ── Event binding ──

function bindEvents() {
  $addBtn.addEventListener("click", handleAddFiles);
  $removeBtn.addEventListener("click", handleRemoveSelected);
  $formatSelect.addEventListener("change", () => {
    state.format = $formatSelect.value as RenameFormat;
    refreshPreview();
  });
  $chkBackup.addEventListener("change", () => {
    state.createBackup = $chkBackup.checked;
  });
  $chkHeic.addEventListener("change", () => {
    state.convertHeic = $chkHeic.checked;
    refreshPreview();
  });
  $offsetSeconds.addEventListener("input", () => {
    state.offsetSeconds = parseInt($offsetSeconds.value, 10) || 0;
    refreshPreview();
  });
  $expandBtn.addEventListener("click", toggleOffsetExpanded);

  const expandedInputs = [$offYears, $offMonths, $offDays, $offHours, $offMinutes, $offSecs];
  expandedInputs.forEach((el) =>
    el.addEventListener("input", syncExpandedToSeconds)
  );

  $chkAll.addEventListener("change", () => {
    if ($chkAll.checked) {
      state.files.forEach((_, i) => state.selectedIndices.add(i));
    } else {
      state.selectedIndices.clear();
    }
    renderFileList();
    updateRemoveButton();
  });

  $renameBtn.addEventListener("click", handleRename);
  $undoBtn.addEventListener("click", handleUndo);
  $aboutBtn.addEventListener("click", openAboutDialog);
  $closeAbout.addEventListener("click", closeAboutDialog);
  $closeErrors.addEventListener("click", () => $errorPanel.classList.add("hidden"));
  $aboutDialog.addEventListener("click", (event) => {
    if (event.target === $aboutDialog) {
      closeAboutDialog();
    }
  });
  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape" && !$aboutDialog.classList.contains("hidden")) {
      closeAboutDialog();
    }
  });

  listen<ProgressPayload>("rename-progress", (event) => {
    const { current, total, filename } = event.payload;
    const pct = Math.round((current / total) * 100);
    $progressFill.style.width = `${pct}%`;
    $progressText.textContent = `${current} / ${total}`;
    $progressFile.textContent = filename;
  });
}

function setupDragDrop() {
  const appWindow = getCurrentWebviewWindow();
  appWindow.onDragDropEvent(async (event) => {
    if (event.payload.type === "over" || event.payload.type === "enter") {
      $fileListContainer.classList.add("drag-over");
    } else if (event.payload.type === "leave") {
      $fileListContainer.classList.remove("drag-over");
    } else if (event.payload.type === "drop") {
      $fileListContainer.classList.remove("drag-over");
      if (event.payload.paths && event.payload.paths.length > 0) {
        await addPaths(event.payload.paths);
      }
    }
  });
}

// ── Handlers ──

async function handleAddFiles() {
  const selected = await open({
    multiple: true,
    directory: false,
    filters: [
      { name: "Media Files", extensions: MEDIA_EXTENSIONS },
    ],
  });
  if (selected) {
    const paths = Array.isArray(selected) ? selected : [selected];
    await addPaths(paths);
  }
}

async function addPaths(paths: string[]) {
  $statusText.textContent = "Scanning files…";
  try {
    const entries = await invoke<FileEntry[]>("scan_files", { paths });
    const existingPaths = new Set(state.files.map((f) => f.path));
    const newEntries = entries.filter((e) => !existingPaths.has(e.path));
    state.files.push(...newEntries);
    await refreshPreview();
    $statusText.textContent = `${state.files.length} files loaded`;
  } catch (err) {
    $statusText.textContent = `Error: ${err}`;
  }
  updateView();
}

function handleRemoveSelected() {
  const indices = Array.from(state.selectedIndices).sort((a, b) => b - a);
  for (const idx of indices) {
    state.files.splice(idx, 1);
    state.previews.splice(idx, 1);
  }
  state.selectedIndices.clear();
  refreshPreview();
  updateView();
}

function openAboutDialog() {
  $aboutDialog.classList.remove("hidden");
}

function closeAboutDialog() {
  $aboutDialog.classList.add("hidden");
}

async function handleRename() {
  const filesToRename: FileEntry[] = [];
  for (let i = 0; i < state.files.length; i++) {
    if (fileNeedsProcessing(i)) {
      filesToRename.push(state.files[i]);
    }
  }

  if (filesToRename.length === 0) return;

  if (filesToRename.length > CONFIRM_LARGE_BATCH_THRESHOLD) {
    if (!confirm(`You are about to rename ${filesToRename.length} files. Continue?`)) {
      return;
    }
  }

  state.isProcessing = true;
  $progressOverlay.classList.remove("hidden");
  $progressFill.style.width = "0%";
  $progressText.textContent = "0 / " + filesToRename.length;

  try {
    const result = await invoke<RenameResult>("execute_rename", {
      entries: filesToRename,
      format: state.format,
      offsetSeconds: state.offsetSeconds,
      createBackup: state.createBackup,
      convertHeic: state.convertHeic,
    });

    $progressOverlay.classList.add("hidden");
    state.isProcessing = false;

    $statusText.textContent = `Done: ${result.success_count} renamed, ${result.error_count} errors`;

    if (result.errors.length > 0) {
      showErrors(result.errors);
    }

    state.files = [];
    state.previews = [];
    state.selectedIndices.clear();
    state.canUndo = true;
    updateView();
    updateUndoButton();
  } catch (err) {
    $progressOverlay.classList.add("hidden");
    state.isProcessing = false;
    $statusText.textContent = `Rename failed: ${err}`;
  }
}

async function handleUndo() {
  try {
    const msg = await invoke<string>("undo_last_rename");
    $statusText.textContent = msg;
    state.canUndo = false;
    updateUndoButton();
  } catch (err) {
    $statusText.textContent = `Undo failed: ${err}`;
  }
}

// ── Preview ──

let previewTimeout: ReturnType<typeof setTimeout> | null = null;
let previewRequestCounter = 0;

async function refreshPreview() {
  previewRequestCounter += 1;
  const requestId = previewRequestCounter;

  if (state.files.length === 0) {
    if (previewTimeout) {
      clearTimeout(previewTimeout);
      previewTimeout = null;
    }
    state.previews = [];
    renderFileList();
    updateRenameButton();
    return;
  }

  if (previewTimeout) clearTimeout(previewTimeout);
  previewTimeout = setTimeout(async () => {
    try {
      const previews = await invoke<PreviewEntry[]>("preview_rename", {
        entries: state.files,
        format: state.format,
        offsetSeconds: state.offsetSeconds,
        convertHeic: state.convertHeic,
      });
      if (requestId !== previewRequestCounter) return;
      state.previews = previews;
    } catch {
      if (requestId !== previewRequestCounter) return;
      state.previews = [];
    }
    if (requestId !== previewRequestCounter) return;
    renderFileList();
    updateRenameButton();
  }, PREVIEW_DEBOUNCE_MS);
}

// ── Rendering ──

function renderFileList() {
  $tbody.innerHTML = "";

  for (let i = 0; i < state.files.length; i++) {
    const file = state.files[i];
    const preview = state.previews[i];
    const tr = document.createElement("tr");
    if (state.selectedIndices.has(i)) tr.classList.add("selected");

    const isChecked = state.selectedIndices.has(i) ? "checked" : "";

    const dateText = file.datetime
      ? file.datetime.replace("T", " ")
      : "—";

    const badgeClass = file.date_source;
    const badgeLabel =
      file.date_source === "exif"
        ? "EXIF"
        : file.date_source === "filesystem"
          ? "File"
          : "None";

    const newName = preview ? preview.new_name : "…";
    const nameChanges = preview && preview.new_name !== file.filename;
    const warningHtml =
      preview?.warning
        ? ` <span class="warning-icon" title="${escapeHtml(preview.warning)}">⚠</span>`
        : "";

    tr.innerHTML = `
      <td class="col-check"><input type="checkbox" data-idx="${i}" ${isChecked} /></td>
      <td class="col-num">${i + 1}</td>
      <td class="col-name" title="${escapeHtml(file.filename)}">${escapeHtml(file.filename)}</td>
      <td class="col-date"><span class="date-badge ${badgeClass}">${badgeLabel}</span> ${dateText}</td>
      <td class="col-new${nameChanges ? " new-name-cell" : ""}">${escapeHtml(newName)}${warningHtml}</td>
    `;

    const checkbox = tr.querySelector("input[type=checkbox]") as HTMLInputElement;
    checkbox.addEventListener("change", () => {
      const idx = parseInt(checkbox.dataset.idx!, 10);
      if (checkbox.checked) {
        state.selectedIndices.add(idx);
      } else {
        state.selectedIndices.delete(idx);
      }
      tr.classList.toggle("selected", checkbox.checked);
      updateRemoveButton();
      updateSelectAll();
    });

    $tbody.appendChild(tr);
  }
}

function updateView() {
  const hasFiles = state.files.length > 0;
  $emptyState.classList.toggle("hidden", hasFiles);
  $tableWrap.classList.toggle("hidden", !hasFiles);
  renderFileList();
  updateRenameButton();
  updateRemoveButton();
}

function countFilesToRename(): number {
  let count = 0;
  for (let i = 0; i < state.files.length; i++) {
    if (fileNeedsProcessing(i)) {
      count++;
    }
  }
  return count;
}

function fileNeedsProcessing(index: number): boolean {
  const preview = state.previews[index];
  return Boolean(
    preview &&
      (preview.new_name !== state.files[index].filename || state.offsetSeconds !== 0)
  );
}

function updateRenameButton() {
  const count = countFilesToRename();
  $renameBtn.disabled = count === 0 || state.isProcessing;
  $renameBtn.querySelector("span")!.textContent = `Rename ${count} File${count !== 1 ? "s" : ""}`;
}

function updateRemoveButton() {
  $removeBtn.disabled = state.selectedIndices.size === 0;
}

function updateUndoButton() {
  $undoBtn.disabled = !state.canUndo;
}

function updateSelectAll() {
  $chkAll.checked =
    state.files.length > 0 && state.selectedIndices.size === state.files.length;
}

function showErrors(errors: { filename: string; reason: string }[]) {
  $errorTitle.textContent = `${errors.length} Error${errors.length !== 1 ? "s" : ""}`;
  $errorList.innerHTML = "";
  for (const err of errors) {
    const li = document.createElement("li");
    li.innerHTML = `<span class="err-file">${escapeHtml(err.filename)}</span><span class="err-reason">— ${escapeHtml(err.reason)}</span>`;
    $errorList.appendChild(li);
  }
  $errorPanel.classList.remove("hidden");
}

// ── Offset expanded ──

function secondsToExpanded(totalSeconds: number): void {
  const sign = totalSeconds < 0 ? -1 : 1;
  let rest = Math.abs(totalSeconds);

  const years = Math.floor(rest / (365 * 86400));
  rest -= years * 365 * 86400;
  const months = Math.floor(rest / (30 * 86400));
  rest -= months * 30 * 86400;
  const days = Math.floor(rest / 86400);
  rest -= days * 86400;
  const hours = Math.floor(rest / 3600);
  rest -= hours * 3600;
  const minutes = Math.floor(rest / 60);
  rest -= minutes * 60;
  const secs = rest;

  $offYears.value = (years * sign).toString();
  $offMonths.value = (months * sign).toString();
  $offDays.value = (days * sign).toString();
  $offHours.value = (hours * sign).toString();
  $offMinutes.value = (minutes * sign).toString();
  $offSecs.value = (secs * sign).toString();
}

function toggleOffsetExpanded() {
  const isHidden = $offsetExpanded.classList.contains("hidden");
  if (isHidden) {
    secondsToExpanded(state.offsetSeconds);
  } else {
    syncExpandedToSeconds();
  }
  $offsetExpanded.classList.toggle("hidden");
  $expandBtn.classList.toggle("expanded", isHidden);
}

function syncExpandedToSeconds() {
  const years = parseInt($offYears.value, 10) || 0;
  const months = parseInt($offMonths.value, 10) || 0;
  const days = parseInt($offDays.value, 10) || 0;
  const hours = parseInt($offHours.value, 10) || 0;
  const minutes = parseInt($offMinutes.value, 10) || 0;
  const secs = parseInt($offSecs.value, 10) || 0;

  const total =
    years * 365 * 86400 +
    months * 30 * 86400 +
    days * 86400 +
    hours * 3600 +
    minutes * 60 +
    secs;

  $offsetSeconds.value = total.toString();
  state.offsetSeconds = total;
  refreshPreview();
}

// ── Helpers ──

function escapeHtml(str: string): string {
  const div = document.createElement("div");
  div.textContent = str;
  return div.innerHTML;
}

// ── Boot ──

window.addEventListener("DOMContentLoaded", init);
