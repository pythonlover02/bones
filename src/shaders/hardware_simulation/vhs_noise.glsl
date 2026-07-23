    {
        float vline = floor(v_uv.y * u_resolution.y);
        float vhs_tn = hash21(vec2(vline, floor(u_time * 3.0) + 0.5)) - 0.5;
        c = c + vec3(vhs_tn * 0.09);
        float vn = hash21(v_uv + vec2(u_time * 10.0)) - 0.5;
        c = c + vec3(vn) * 0.07;
        float vhs_dh = hash21(vec2(vline, floor(u_time * 30.0)));
        float vhs_dv = step(0.985, vhs_dh);
        c = mix(c, vec3(step(0.5, fract(vhs_dh * 3.17))), vhs_dv * 0.7);
    }
