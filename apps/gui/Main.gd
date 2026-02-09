extends Control

const FAG_EXE := "res://bin/fag.exe"
const POLL_SECS := 1.0
const MAX_LOG_LINES_PER_TICK := 200
const MAX_EVENT_ROWS := 500

@onready var ext_edit: LineEdit = $VBox/Row/Ext
@onready var name_edit: LineEdit = $VBox/Row/Name
@onready var output: TextEdit = $VBox/Tabs/OutputTab/Output
@onready var interval_spin: SpinBox = $VBox/WatchRow/Interval
@onready var start_watch_btn: Button = $VBox/WatchRow/BtnStartWatch
@onready var stop_watch_btn: Button = $VBox/WatchRow/BtnStopWatch
@onready var watch_status: Label = $VBox/WatchRow/WatchStatus
@onready var tabs: TabContainer = $VBox/Tabs

@onready var refresh_rules_btn: Button = $VBox/Tabs/RulesTab/RulesRow/BtnRefreshRules
@onready var remove_selected_rule_btn: Button = $VBox/Tabs/RulesTab/RulesRow/BtnRemoveSelectedRule
@onready var rules_path_label: Label = $VBox/Tabs/RulesTab/RulesRow/RulesPath
@onready var rules_list: ItemList = $VBox/Tabs/RulesTab/RulesList

@onready var clear_events_btn: Button = $VBox/Tabs/EventsTab/EventsRow/BtnClearEvents
@onready var events_tree: Tree = $VBox/Tabs/EventsTab/EventsTree

var _watch_pid: int = -1
var _log_path: String = ""
var _log_offset: int = 0
var _poll_timer: Timer
var _events_root: TreeItem
var _rules_items: Array[Dictionary] = []
var _event_items: Array = []

func _ready() -> void:
	$VBox/Row2/BtnSysinfo.pressed.connect(_on_sysinfo)
	$VBox/Row2/BtnLatest.pressed.connect(_on_latest)
	$VBox/Row2/BtnProgids.pressed.connect(_on_progids)
	$VBox/Row2/BtnCapture.pressed.connect(_on_capture)
	$VBox/Row2/BtnApply.pressed.connect(_on_apply)
	$VBox/Row2/BtnAddRule.pressed.connect(_on_rule_add)
	$VBox/Row2/BtnRemoveRule.pressed.connect(_on_rule_remove)
	$VBox/Row2/BtnCheck.pressed.connect(_on_check)
	start_watch_btn.pressed.connect(_on_start_watch_rules)
	stop_watch_btn.pressed.connect(_on_stop_watch_rules)
	refresh_rules_btn.pressed.connect(_refresh_rules)
	remove_selected_rule_btn.pressed.connect(_remove_selected_rule)
	clear_events_btn.pressed.connect(_clear_events)
	rules_list.item_selected.connect(_on_rule_selected)

	tabs.set_tab_title(0, "Output")
	tabs.set_tab_title(1, "Rules")
	tabs.set_tab_title(2, "Events")

	events_tree.clear()
	_events_root = events_tree.create_item()

	_log_path = _default_log_path()
	_init_log_offset()
	_poll_timer = Timer.new()
	_poll_timer.wait_time = POLL_SECS
	_poll_timer.one_shot = false
	_poll_timer.timeout.connect(_on_poll)
	add_child(_poll_timer)
	_poll_timer.start()

	_write("GUI ready.")
	_write("If res://bin/fag.exe is missing, run: scripts/build-gui.ps1")
	_write("Log: " + _log_path)
	_refresh_rules()

func _on_sysinfo() -> void:
	_run_and_show(["sysinfo"])

func _on_latest() -> void:
	_run_and_show(["latest", "--ext", ext_edit.text])

func _on_progids() -> void:
	_run_and_show(["progids", "--ext", ext_edit.text])

func _on_capture() -> void:
	_run_and_show(["capture-latest", "--ext", ext_edit.text, "--name", name_edit.text])

func _on_apply() -> void:
	_run_and_show(["apply-latest", "--ext", ext_edit.text, "--name", name_edit.text])

func _on_rule_add() -> void:
	_run_and_show(["rules", "add", "--ext", ext_edit.text, "--name", name_edit.text])
	_refresh_rules()

func _on_rule_remove() -> void:
	_run_and_show(["rules", "remove", "--ext", ext_edit.text])
	_refresh_rules()

func _on_check() -> void:
	_run_and_show(["check"])

func _on_start_watch_rules() -> void:
	if _is_watch_running():
		_write("Watch already running (pid=" + str(_watch_pid) + ")")
		return

	var exe := ProjectSettings.globalize_path(FAG_EXE)
	if !FileAccess.file_exists(exe):
		_write("ERROR: missing " + exe)
		return

	var interval := int(interval_spin.value)
	if interval <= 0:
		interval = 5

	var args: PackedStringArray = ["watch-rules", "--interval", str(interval)]
	var pid := OS.create_process(exe, args, false)
	if pid <= 0:
		_write("ERROR: failed to start watch-rules")
		return

	_watch_pid = pid
	_update_watch_ui()
	_write("$ " + exe + " " + " ".join(args))
	_write("started pid=" + str(_watch_pid))

func _on_stop_watch_rules() -> void:
	if !_is_watch_running():
		_watch_pid = -1
		_update_watch_ui()
		_write("Watch already stopped.")
		return

	OS.kill(_watch_pid)
	_write("stopped pid=" + str(_watch_pid))
	_watch_pid = -1
	_update_watch_ui()

func _on_poll() -> void:
	_update_watch_ui()
	if _is_watch_running() or tabs.current_tab == 2:
		_tail_log()

func _update_watch_ui() -> void:
	var running := _is_watch_running()
	start_watch_btn.disabled = running
	stop_watch_btn.disabled = !running
	watch_status.text = "running (pid=" + str(_watch_pid) + ")" if running else "stopped"

func _is_watch_running() -> bool:
	return _watch_pid > 0 and OS.is_process_running(_watch_pid)

func _tail_log() -> void:
	if _log_path.is_empty():
		return
	if !FileAccess.file_exists(_log_path):
		return

	var f = FileAccess.open(_log_path, FileAccess.READ)
	if f == null:
		return

	var size := f.get_length()
	if _log_offset > size:
		_log_offset = 0
	f.seek(_log_offset)
	var processed := 0
	while processed < MAX_LOG_LINES_PER_TICK and !f.eof_reached():
		var line := f.get_line().strip_edges()
		_log_offset = f.get_position()
		if line.is_empty():
			continue
		_append_event_line(line)
		processed += 1

func _default_log_path() -> String:
	var appdata := OS.get_environment("APPDATA")
	if appdata.is_empty():
		return ""
	return appdata.path_join("FileAssocGuard").path_join("guard.log")

func _init_log_offset() -> void:
	if _log_path.is_empty():
		_log_offset = 0
		return
	if !FileAccess.file_exists(_log_path):
		_log_offset = 0
		return
	var f = FileAccess.open(_log_path, FileAccess.READ)
	if f == null:
		_log_offset = 0
		return
	var size := f.get_length()
	# Avoid loading huge history into the UI on startup.
	var max_bytes := 64 * 1024
	_log_offset = int(max(size - max_bytes, 0))

func _run_and_show(args: Array[String]) -> void:
	var exe := ProjectSettings.globalize_path(FAG_EXE)
	if !FileAccess.file_exists(exe):
		_write("ERROR: missing " + exe)
		return

	var out: Array = []
	var code := OS.execute(exe, PackedStringArray(args), out, true, false)
	var text := ""
	for chunk in out:
		text += str(chunk)
	text = text.strip_edges()

	_write("$ " + exe + " " + " ".join(args))
	_write("exit=" + str(code))
	if text.is_empty():
		_write("(no output)")
		return
	_write(text)

func _write(line: String) -> void:
	output.text += line + "\n"
	output.scroll_vertical = output.get_line_count()

func _refresh_rules() -> void:
	var exe := ProjectSettings.globalize_path(FAG_EXE)
	if !FileAccess.file_exists(exe):
		rules_path_label.text = "missing backend"
		return

	var out: Array = []
	var code := OS.execute(exe, PackedStringArray(["rules", "list"]), out, true, false)
	var text := ""
	for chunk in out:
		text += str(chunk)
	text = text.strip_edges()

	if code != 0:
		rules_path_label.text = "error"
		_write("$ " + exe + " rules list")
		_write("exit=" + str(code))
		_write(text)
		return

	var data = _parse_json(text)
	if data == null or typeof(data) != TYPE_DICTIONARY:
		rules_path_label.text = "parse error"
		return

	var rules_path := str(data.get("rules_path", ""))
	rules_path_label.text = rules_path
	_rules_items.clear()
	rules_list.clear()

	var rules_arr = data.get("rules", [])
	if typeof(rules_arr) != TYPE_ARRAY:
		return

	for r in rules_arr:
		if typeof(r) != TYPE_DICTIONARY:
			continue
		var ext := str(r.get("ext", ""))
		var name := str(r.get("name", ""))
		if ext.is_empty() or name.is_empty():
			continue
		_rules_items.append({"ext": ext, "name": name})
		rules_list.add_item(ext + " -> " + name)

func _on_rule_selected(index: int) -> void:
	if index < 0 or index >= _rules_items.size():
		return
	var item: Dictionary = _rules_items[index]
	ext_edit.text = str(item.get("ext", ext_edit.text))
	name_edit.text = str(item.get("name", name_edit.text))

func _remove_selected_rule() -> void:
	var selected := rules_list.get_selected_items()
	if selected.size() == 0:
		_write("No rule selected.")
		return
	var idx := int(selected[0])
	if idx < 0 or idx >= _rules_items.size():
		return
	var ext := str(_rules_items[idx].get("ext", ""))
	if ext.is_empty():
		return
	_run_and_show(["rules", "remove", "--ext", ext])
	_refresh_rules()

func _clear_events() -> void:
	events_tree.clear()
	_events_root = events_tree.create_item()
	_event_items.clear()
	_write("Events cleared.")

func _append_event_line(line: String) -> void:
	var data = _parse_json(line)
	if data == null or typeof(data) != TYPE_DICTIONARY:
		return

	var ms := int(data.get("time_unix_ms", 0))
	var ext := str(data.get("ext", ""))
	var name := str(data.get("name", ""))
	var status := str(data.get("status", ""))
	var effective := str(data.get("effective_progid", ""))
	var target := str(data.get("target_progid", ""))

	var item := events_tree.create_item(_events_root)
	item.set_text(0, _format_time(ms))
	item.set_text(1, ext)
	item.set_text(2, name)
	item.set_text(3, status)
	item.set_text(4, effective)
	item.set_text(5, target)
	_event_items.append(item)
	if _event_items.size() > MAX_EVENT_ROWS:
		var old = _event_items[0]
		_event_items.remove_at(0)
		if old != null:
			old.free()

func _format_time(ms: int) -> String:
	if ms <= 0:
		return ""
	var d = Time.get_datetime_dict_from_unix_time(int(ms / 1000))
	return "%04d-%02d-%02d %02d:%02d:%02d" % [d.year, d.month, d.day, d.hour, d.minute, d.second]

func _parse_json(text: String) -> Variant:
	if text.is_empty():
		return null
	return JSON.parse_string(text)

func _notification(what: int) -> void:
	if what == NOTIFICATION_WM_CLOSE_REQUEST:
		if _is_watch_running():
			OS.kill(_watch_pid)
		get_tree().quit()
