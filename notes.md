# Structure

- Create ReignHandle
- Attach systems: update, render, etc.
- Run: calls run_app

- Systems are scheduled in RedrawRequested event

- ECS for game specific components 

# Components

## ECS

## Renderer

## Resource Manager

- Load textures and audio
- Maintain textures and audio

## Audio

## Utilities

## Texture loading

- Call load texture whenever
- Texture is resized
    - Double dimensions until one or both is greater then either 256x256 or 4096x4096
    - Resize to dimension step right before
    - Fill remaining space with alpha pixels
- When draw texture is called the texture is written to the queue on that frame in the next open spot
    - Draw pass tracks next available texture slot
    - When called uv is calculated based on ratio of actual texture size to scale

## Sprites

- Shader
    - Camera uniform
    - Quad vertex buffer
        - Contains
    - Instance buffer
        - Stores texture index, world position, uv, etc.
    - Sprites are drawn using wgpu instances

## Enemy AI

- Basic Enemies
    - Component/System Based
    - Add and remove components based on the current state
        - Ex: Idle until sees player, remove idle and add tracking
- Bosses
    - Boss state machine component


## Roadmap to first boss

- Enemies
    - Deer
    - Owl
    - Bear

- Boss
    - Giant owl

# Tiles/Objects

- Tiles
    - Grid alligned
    - Simple/cheap
    - Has
        - Type
            - id
                - u32
        - Other state
            - u32
    - Data
        - Name
        - Properties
        - Rendering info
        - Collision info
    - Use for
        - Grid alligned
        - Simple state
- Objects
    - Optionally grid alligned
    - More unique behaviors
    - Has
        - Type
            - id
                - u32
        - Position
        - Entity option
            - Can store more complex information, ex: inventories
        - Other state
    - Use for
        - Non grid alligned
        - More complex objects
        - Things which require the ecs