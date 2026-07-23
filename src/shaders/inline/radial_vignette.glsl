    {
        vec2 vd = v_uv - HALF2;
        c = c * (1.0 - smoothstep(0.09, 0.64, dot(vd, vd)) * 0.5);
    }
