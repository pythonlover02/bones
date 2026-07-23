    {
        float cp = fract(v_uv.x * u_resolution.x * 0.3333333);
        float mr = mix(0.7, 1.0, step(cp, 0.3333333));
        float mg = mix(0.7, 1.0, step(0.3333333, cp) * step(cp, 0.6666666));
        float mb = mix(0.7, 1.0, step(0.6666666, cp));
        float sc = 1.0 - 0.25 * (0.5 + 0.5 * (abs(fract(v_uv.y * u_resolution.y * 0.5) - 0.5) * 4.0 - 1.0));
        c = c * vec3(mr, mg, mb) * sc * 1.08;
    }
