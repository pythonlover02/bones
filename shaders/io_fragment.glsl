layout(set=0, binding=0) uniform sampler2D u_input;
layout(set=0, binding=1) uniform sampler2D u_history;
layout(push_constant) uniform PushBlock { vec2 res; float time; float fps; } pc;
#define u_resolution pc.res
#define u_time pc.time
#define u_fps pc.fps
layout(location=0) out vec4 frag_out;
#define BONES_FRAGCOORD gl_FragCoord.xy
#define BONES_WRITE_OUT(rgb) frag_out = vec4(clamp(rgb, 0.0, 1.0), 1.0)
#define BONES_EARLY_OUT
