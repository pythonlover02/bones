    {
        vec2 cc2 = v_uv - HALF2;
        float cr = dot(cc2, cc2);
        v_uv = v_uv + cc2 * cr * vec2(0.031, 0.041);
    }
