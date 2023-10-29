extends CharacterBody3D

# Get the gravity from the project settings to be synced with RigidBody nodes.
var gravity = ProjectSettings.get_setting("physics/3d/default_gravity")

@export
var player = 0
@export
var controlled = true:
	set(value):
		$Camera3D.current = value
		controlled = value

func _ready():
	Input.mouse_mode = Input.MOUSE_MODE_CAPTURED
