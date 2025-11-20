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
