extends CharacterBody3D


const SPEED = 5.0
const JUMP_VELOCITY = 4.5

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

func _input(event):
	if event is InputEventMouseMotion:
		var rotation_x = -event.relative.x * 0.005
		var rotation_y = -event.relative.y * 0.005
		rotate(Vector3.UP, rotation_x)
		$Camera3D.rotation.x = clamp($Camera3D.rotation.x + rotation_y, -PI/2, PI/2)

func _physics_process(delta):
	if not controlled:
		return
	
	# Add the gravity.
	if not is_on_floor():
		velocity.y -= gravity * delta

	# Handle Jump.
	if Input.is_action_just_pressed("g_jump") and is_on_floor():
		velocity.y = JUMP_VELOCITY

	# Get the input direction and handle the movement/deceleration.
	# As good practice, you should replace UI actions with custom gameplay actions.
	var input_dir = Input.get_vector("g_left", "g_right", "g_forward", "g_back")
	var direction = (transform.basis * Vector3(input_dir.x, 0, input_dir.y)).normalized()
	if direction:
		velocity.x = direction.x * SPEED
		velocity.z = direction.z * SPEED
	else:
		velocity.x = move_toward(velocity.x, 0, SPEED)
		velocity.z = move_toward(velocity.z, 0, SPEED)

	move_and_slide()

	if position.y < -100:
		position.y = 100
	
	$/root/GameClass.update_player_position(position)
