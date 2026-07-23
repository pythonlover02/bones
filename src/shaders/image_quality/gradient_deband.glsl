    {
        float dr = 8.0 * res_scale;
        float dh1 = hash21(v_uv);
        float dh2 = hash21(v_uv + 17.31);
        vec2 ddir = vec2(dh1, dh2) * 2.0 - 1.0;
        ddir = ddir / max(length(ddir), 0.0001);
        vec3 ds = texture(u_input, v_uv + ddir * dr * inv).rgb;
        vec3 dd = abs(ds - c);
        float dm = step(max(dd.r, max(dd.g, dd.b)), 0.02);
        c = mix(c, (c + ds) * 0.5, dm);
    }
