    {
        vec2 dc = v_uv - HALF2;
        v_uv = HALF2 + dc * (1.0 + 0.1 * dot(dc, dc));
    }
