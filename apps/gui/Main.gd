extends Control

const FAG_EXE := "res://bin/fag.exe"

@onready var ext_edit: LineEdit = $VBox/Row/Ext
@onready var name_edit: LineEdit = $VBox/Row/Name
@onready var output: TextEdit = $VBox/Output

func _ready() -> void:
	$VBox/Row2/BtnSysinfo.pressed.connect(_on_sysinfo)
	$VBox/Row2/BtnLatest.pressed.connect(_on_latest)
	$VBox/Row2/BtnProgids.pressed.connect(_on_progids)
	$VBox/Row2/BtnCapture.pressed.connect(_on_capture)
	$VBox/Row2/BtnApply.pressed.connect(_on_apply)
	$VBox/Row2/BtnAddRule.pressed.connect(_on_rule_add)
	$VBox/Row2/BtnRemoveRule.pressed.connect(_on_rule_remove)
	$VBox/Row2/BtnCheck.pressed.connect(_on_check)

	_write("GUI ready.")
	_write("If res://bin/fag.exe is missing, run: scripts/build-gui.ps1")

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

