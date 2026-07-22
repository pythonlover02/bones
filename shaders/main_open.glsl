void main() {
    BONES_EARLY_OUT;
    vec2 frag_coord = BONES_FRAGCOORD;
    vec2 inv = 1.0 / u_resolution;
    vec2 v_uv = frag_coord * inv;
    float res_scale = u_resolution.y * 0.0009259259;
