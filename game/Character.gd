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
		$Camera3D/RayCast3D.enabled = value

func _ready():
	Input.mouse_mode = Input.MOUSE_MODE_CAPTURED
	$Camera3D/RayCast3D.target_position = Vector3(0, -10, 0)
	
