    {
        float tr = 2.0 * res_scale;
        float td = clamp((abs(v_uv.y - 0.5) - 0.15) / 0.2, 0.0, 1.0);
        float tx = tr * td * inv.x;
        vec3 ts = c
            + texture(u_input, v_uv + vec2(tx, 0.0)).rgb
            + texture(u_input, v_uv - vec2(tx, 0.0)).rgb
            + texture(u_input, v_uv + vec2(tx * 2.0, 0.0)).rgb
            + texture(u_input, v_uv - vec2(tx * 2.0, 0.0)).rgb
            + texture(u_input, v_uv + vec2(tx * 3.0, 0.0)).rgb
            + texture(u_input, v_uv - vec2(tx * 3.0, 0.0)).rgb
            + texture(u_input, v_uv + vec2(tx * 4.0, 0.0)).rgb
            + texture(u_input, v_uv - vec2(tx * 4.0, 0.0)).rgb;
        c = mix(c, ts * 0.1111111, td);
    }
