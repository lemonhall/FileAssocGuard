extends Control

const FAG_EXE := "res://bin/fag.exe"

@onready var ext_edit: LineEdit = $VBox/Row/Ext
@onready var name_edit: LineEdit = $VBox/Row/Name
@onready var output: TextEdit = $VBox/Output
@onready var interval_spin: SpinBox = $VBox/WatchRow/Interval
@onready var start_watch_btn: Button = $VBox/WatchRow/BtnStartWatch
@onready var stop_watch_btn: Button = $VBox/WatchRow/BtnStopWatch
@onready var watch_status: Label = $VBox/WatchRow/WatchStatus

var _watch_pid: int = -1
var _log_path: String = ""
var _log_offset: int = 0
var _poll_timer: Timer

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

	_log_path = _default_log_path()
	_poll_timer = Timer.new()
	_poll_timer.wait_time = 0.5
	_poll_timer.one_shot = false
	_poll_timer.timeout.connect(_on_poll)
	add_child(_poll_timer)
	_poll_timer.start()

	_write("GUI ready.")
	_write("If res://bin/fag.exe is missing, run: scripts/build-gui.ps1")
	_write("Log: " + _log_path)

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

func _on_rule_remove() -> void:
	_run_and_show(["rules", "remove", "--ext", ext_edit.text])

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

	var f := FileAccess.open(_log_path, FileAccess.READ)
	if f == null:
		return

	var size := f.get_length()
	if _log_offset > size:
		_log_offset = 0
	f.seek(_log_offset)
	var text := f.get_as_text()
	_log_offset = f.get_position()
	if text.is_empty():
		return
	for line in text.split("\n", false):
		if line.strip_edges().is_empty():
			continue
		_write("[log] " + line)

func _default_log_path() -> String:
	var appdata := OS.get_environment("APPDATA")
	if appdata.is_empty():
		return ""
	return appdata.path_join("FileAssocGuard").path_join("guard.log")

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

func _notification(what: int) -> void:
	if what == NOTIFICATION_WM_CLOSE_REQUEST:
		if _is_watch_running():
			OS.kill(_watch_pid)
		get_tree().quit()
