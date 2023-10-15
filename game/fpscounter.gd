extends Label

func _process(delta: float) -> void:
	set_text("fps: %d" % Engine.get_frames_per_second())
