layout(local_size_x = LOCAL_SIZE_X, local_size_y = LOCAL_SIZE_Y, local_size_z = 1) in;
layout(set=0, binding=0) uniform sampler2D u_input;
layout(set=0, binding=1) uniform sampler2D u_history;
layout(set=0, binding=2, rgba8) uniform writeonly image2D u_output;
layout(push_constant) uniform PushBlock { vec2 res; float time; float fps; } pc;
#define u_resolution pc.res
#define u_time pc.time
#define u_fps pc.fps
#define BONES_FRAGCOORD vec2(gl_GlobalInvocationID.xy) + vec2(0.5)
#define BONES_WRITE_OUT(rgb) imageStore(u_output, ivec2(gl_GlobalInvocationID.xy), vec4(clamp(rgb, 0.0, 1.0), 1.0))
#define BONES_EARLY_OUT if (any(greaterThanEqual(gl_GlobalInvocationID.xy, uvec2(u_resolution)))) return
