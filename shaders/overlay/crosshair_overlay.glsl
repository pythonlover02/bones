    {
        float cs = res_scale;
        vec2 cp = abs(frag_coord - u_resolution * 0.5);
        float in_box = step(max(cp.x, cp.y), 16.0 * cs);
        float d_dot = max(0.0, length(cp) - 0.5 * cs);
        vec2 pa_h = cp - vec2(4.0 * cs, 0.0);
        float h_h = clamp(pa_h.x / (10.0 * cs), 0.0, 1.0);
        float d_arm_h = length(pa_h - vec2(10.0 * cs * h_h, 0.0));
        vec2 pa_v = cp - vec2(0.0, 4.0 * cs);
        float h_v = clamp(pa_v.y / (10.0 * cs), 0.0, 1.0);
        float d_arm_v = length(pa_v - vec2(0.0, 10.0 * cs * h_v));
        float d_cross = min(d_dot, min(d_arm_h, d_arm_v));
        float cross_core = (1.0 - smoothstep(0.5 * cs, 1.5 * cs, d_cross)) * in_box;
        float cross_glow = (1.0 - smoothstep(1.0 * cs, 4.0 * cs, d_cross)) * in_box;
        c = mix(c, vec3(0.0, 0.5, 1.0), cross_glow * 0.85);
        c = mix(c, vec3(0.85, 0.95, 1.0), cross_core);
    }
