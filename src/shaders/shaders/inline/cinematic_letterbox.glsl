    {
        float la = u_resolution.x / max(u_resolution.y, 0.0001);
        float lv = clamp(la * 0.42553191, 0.0, 1.0);
        float lb = (1.0 - lv) * 0.5;
        c = mix(c, ZERO3, step(v_uv.y, lb) + step(1.0 - lb, v_uv.y));
    }
